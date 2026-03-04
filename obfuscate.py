import hashlib
import random
import string
from dispatcher import *
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
    
    def renameLabel(self,label):
        target = hashlib.sha256(label.encode('utf-8')).hexdigest()
        for idx,ins in enumerate(self.instructions):
            self.instructions[idx] = Instruction(ins.rawLine.replace(label,target))
        for idx,oldLabel in enumerate(self.labels):
            if oldLabel == label:
                self.labels[idx] = target

    def renameAllLabels(self):
        for label in self.labels:
            self.renameLabel(label)

    def generate_dispatchers(self):
        pass
    
    def swapCallToDispatcher(self):
        pass



                
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
    #TheCode.renameAllLabels()
    TheCode.saveToFile()
    #print(table)