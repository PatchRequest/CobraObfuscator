import hashlib
import random
import string

class Code:
    def __init__(self):
        self.instructions = []
        self.labels = []

    def appendInstruction(self,instruction):
        if not isinstance(instruction,Instruction) or instruction.__dict__ == {}:
            return
        self.instructions.append(instruction)
        if instruction.is_label:
            self.labels.append(instruction.labelContent)
        
    def printAll(self):
        for ins in self.instructions:
            print(ins)

    def accessSection(self,section):
        if not section in self.sections.keys():
            raise Exception(f'Illegal Section Access {section}')
        return self.sections[section]
    
    def renameLabel(self,label):
        target = hashlib.sha256(label.encode('utf-8')).hexdigest()
        for idx,ins in enumerate(self.instructions):
            self.instructions[idx] = Instruction(ins.rawLine.replace(label,target))
        for idx,oldLabel in enumerate(self.labels):
            if oldLabel == label:
                self.labels[idx] = target

    def renameAllLabers(self):
        for label in self.labels:
            self.renameLabel(label)

    def generate_dispatchers(self,num_dispatchers=2, proxy_chance=0.5):
        labels = self.labels
        label_to_dispatch_info = {}
        dispatchers = []
        proxies = []
        dispatcher_labels = [f"dispatcher_{i}" for i in range(num_dispatchers)]
        
        # Group labels into dispatchers
        grouped_labels = [[] for _ in range(num_dispatchers)]
        for label in labels:
            dispatcher_idx = random.randint(0, num_dispatchers - 1)
            grouped_labels[dispatcher_idx].append(label)
        
        code_lines = []
        
        for i, dispatcher in enumerate(dispatcher_labels):
            code_lines.append(f"{dispatcher}:")
            used_eax = set()
            for label in grouped_labels[i]:
                # Random EAX value that hasn't been used in this dispatcher
                eax_val = random.randint(1, 100)
                while eax_val in used_eax:
                    eax_val = random.randint(1, 100)
                used_eax.add(eax_val)

                # Decide if proxy is used
                if random.random() < proxy_chance:
                    proxy_label = ''.join(random.choices(string.ascii_lowercase, k=12))
                    code_lines.append(f"    cmp eax, {eax_val}")
                    code_lines.append(f"    je {proxy_label}")
                    proxies.append((proxy_label, label))
                    label_to_dispatch_info[label] = (dispatcher, eax_val)
                else:
                    code_lines.append(f"    cmp eax, {eax_val}")
                    code_lines.append(f"    je {label}")
                    label_to_dispatch_info[label] = (dispatcher, eax_val)
            code_lines.append(f"    ret\n")

        # Add proxy functions
        for proxy_label, target_label in proxies:
            code_lines.append(f"{proxy_label}:")
            code_lines.append(f"    jmp {target_label}\n")

        for label in label_to_dispatch_info.keys():
            self.swapCallToDispatcher(label,label_to_dispatch_info)

        for line in code_lines:
            self.appendInstruction(Instruction(line))
        #return label_to_dispatch_info

    def swapCallToDispatcher(self,label,info_table):
        #print(f"[+] Swapping Calls to Dispatcher for {label} ")
        insertions = []

        for idx, ins in enumerate(self.instructions):
            if label in ins.rawLine and not ins.is_label:
                insertions.append((idx, ins))

        # Apply in reverse to keep indices stable
        for idx, ins in reversed(insertions):
            code_lines = [
                Instruction("push eax"),
                Instruction(f"mov eax, {info_table[label][1]}"),
                Instruction(f"{ins.op} {info_table[label][0]}"),
                Instruction("pop eax")
            ]
            self.instructions[idx:idx+1] = code_lines

                
    def saveToFile(self):
        with open("output.asm", 'w') as f:
            for ins in self.instructions:
                f.write(f"{ins}\n")

class Instruction:
    def __init__(self,raw):
        stripped = raw.strip()
        if stripped == "":
            return None
        if stripped[0] == ';':
            return None
        self.rawLine = raw.strip()
        self.parts = [x.strip() for x in self.rawLine.split(" ") if x]
        self.op = self.parts[0]
        self.is_label = False
        self.multi_line = False
        self.is_dispatcher_call = "dispatcher_" in self.rawLine
        if self.op[-1] == ':':
            self.labelContent = self.rawLine[:-1]
            self.is_label = True
        if self.parts[-1] == "proc" or self.parts[-1] ==  "endp":
            if self.parts[0] == "extern":
                self.labelContent = self.parts[1]
                self.is_label = True
            else:
                self.labelContent = self.parts[0]
                self.is_label = True
        
    def __repr__(self):
        return self.rawLine

TheCode = Code()
with open("code.asm","r+") as input:
    lines = input.readlines()
    for line in lines:
        ins = Instruction(line)
        TheCode.appendInstruction(ins)
    #TheCode.groupSections()
    #TheCode.renameAllLabers()
    


    TheCode.generate_dispatchers()
    TheCode.saveToFile()
    #print(table)