// Go test program: algorithms, goroutines, channels, interfaces
// Tests obfuscator against Go's unique code generation (goroutine stacks,
// GC cooperation points, interface dispatch, defer/panic/recover)
package main

import (
	"fmt"
	"math"
	"os"
	"runtime"
	"sort"
	"strings"
	"sync"
	"sync/atomic"
)

var pass, fail int32

func check(cond bool, msg string) {
	if cond {
		fmt.Printf("  [OK] %s\n", msg)
		atomic.AddInt32(&pass, 1)
	} else {
		fmt.Printf("  [FAIL] %s\n", msg)
		atomic.AddInt32(&fail, 1)
	}
}

// --- Mergesort ---
func mergesort(arr []int) []int {
	if len(arr) <= 1 {
		return arr
	}
	mid := len(arr) / 2
	left := mergesort(arr[:mid])
	right := mergesort(arr[mid:])
	return merge(left, right)
}

func merge(a, b []int) []int {
	result := make([]int, 0, len(a)+len(b))
	i, j := 0, 0
	for i < len(a) && j < len(b) {
		if a[i] <= b[j] {
			result = append(result, a[i])
			i++
		} else {
			result = append(result, b[j])
			j++
		}
	}
	result = append(result, a[i:]...)
	result = append(result, b[j:]...)
	return result
}

// --- Heap (priority queue) ---
type MinHeap struct {
	data []int
}

func (h *MinHeap) Push(val int) {
	h.data = append(h.data, val)
	i := len(h.data) - 1
	for i > 0 {
		parent := (i - 1) / 2
		if h.data[parent] <= h.data[i] {
			break
		}
		h.data[parent], h.data[i] = h.data[i], h.data[parent]
		i = parent
	}
}

func (h *MinHeap) Pop() int {
	val := h.data[0]
	n := len(h.data) - 1
	h.data[0] = h.data[n]
	h.data = h.data[:n]
	i := 0
	for {
		left := 2*i + 1
		right := 2*i + 2
		smallest := i
		if left < n && h.data[left] < h.data[smallest] {
			smallest = left
		}
		if right < n && h.data[right] < h.data[smallest] {
			smallest = right
		}
		if smallest == i {
			break
		}
		h.data[i], h.data[smallest] = h.data[smallest], h.data[i]
		i = smallest
	}
	return val
}

// --- Interface dispatch ---
type Shape interface {
	Area() float64
	Name() string
}

type Circle struct{ Radius float64 }
type Rectangle struct{ W, H float64 }
type Triangle struct{ Base, Height float64 }

func (c Circle) Area() float64    { return math.Pi * c.Radius * c.Radius }
func (c Circle) Name() string     { return "circle" }
func (r Rectangle) Area() float64 { return r.W * r.H }
func (r Rectangle) Name() string  { return "rectangle" }
func (t Triangle) Area() float64  { return 0.5 * t.Base * t.Height }
func (t Triangle) Name() string   { return "triangle" }

// --- Concurrency: parallel map ---
func parallelMap(data []int, fn func(int) int) []int {
	result := make([]int, len(data))
	var wg sync.WaitGroup
	for i, v := range data {
		wg.Add(1)
		go func(idx, val int) {
			defer wg.Done()
			result[idx] = fn(val)
		}(i, v)
	}
	wg.Wait()
	return result
}

// --- Channel pipeline ---
func generator(nums ...int) <-chan int {
	out := make(chan int)
	go func() {
		for _, n := range nums {
			out <- n
		}
		close(out)
	}()
	return out
}

func square(in <-chan int) <-chan int {
	out := make(chan int)
	go func() {
		for n := range in {
			out <- n * n
		}
		close(out)
	}()
	return out
}

func filterEven(in <-chan int) <-chan int {
	out := make(chan int)
	go func() {
		for n := range in {
			if n%2 == 0 {
				out <- n
			}
		}
		close(out)
	}()
	return out
}

// --- Defer/panic/recover ---
func safeDiv(a, b int) (result int, err string) {
	defer func() {
		if r := recover(); r != nil {
			err = fmt.Sprintf("%v", r)
			result = -1
		}
	}()
	if b == 0 {
		panic("division by zero")
	}
	return a / b, ""
}

// --- Map operations ---
func wordFrequency(text string) map[string]int {
	freq := make(map[string]int)
	for _, word := range strings.Fields(text) {
		freq[strings.ToLower(word)]++
	}
	return freq
}

// --- Recursive fibonacci with memoization via closure ---
func makeFibMemo() func(int) int64 {
	memo := map[int]int64{0: 0, 1: 1}
	var fib func(int) int64
	fib = func(n int) int64 {
		if v, ok := memo[n]; ok {
			return v
		}
		memo[n] = fib(n-1) + fib(n-2)
		return memo[n]
	}
	return fib
}

func main() {
	fmt.Println("=== Go Algorithm/Concurrency Tests ===")

	// Mergesort
	data := []int{9, 3, 7, 1, 8, 2, 6, 4, 5, 0}
	sorted := mergesort(data)
	isSorted := true
	for i := 0; i < len(sorted); i++ {
		if sorted[i] != i {
			isSorted = false
		}
	}
	check(isSorted, "mergesort")

	// Heap
	heap := &MinHeap{}
	for _, v := range []int{5, 3, 8, 1, 9, 2, 7, 4, 6, 0} {
		heap.Push(v)
	}
	heapSorted := true
	for i := 0; i < 10; i++ {
		if heap.Pop() != i {
			heapSorted = false
		}
	}
	check(heapSorted, "min-heap extract order")

	// Interface dispatch
	shapes := []Shape{
		Circle{Radius: 5},
		Rectangle{W: 4, H: 6},
		Triangle{Base: 3, Height: 8},
	}
	check(shapes[0].Name() == "circle", "circle name")
	check(shapes[1].Name() == "rectangle", "rectangle name")
	check(shapes[2].Name() == "triangle", "triangle name")

	circleArea := shapes[0].Area()
	check(math.Abs(circleArea-78.53981633974483) < 0.0001, "circle area")
	check(shapes[1].Area() == 24.0, "rectangle area")
	check(shapes[2].Area() == 12.0, "triangle area")

	// Parallel map
	input := make([]int, 100)
	for i := range input {
		input[i] = i + 1
	}
	squared := parallelMap(input, func(x int) int { return x * x })
	check(squared[0] == 1, "parallel map [0]")
	check(squared[9] == 100, "parallel map [9]")
	check(squared[99] == 10000, "parallel map [99]")
	psum := 0
	for _, v := range squared {
		psum += v
	}
	check(psum == 338350, "parallel map sum of squares")

	// Channel pipeline
	ch := square(generator(1, 2, 3, 4, 5, 6, 7, 8, 9, 10))
	evenCh := filterEven(ch)
	var pipelineSum int
	for v := range evenCh {
		pipelineSum += v
	}
	// squares: 1,4,9,16,25,36,49,64,81,100; even: 4,16,36,64,100 = 220
	check(pipelineSum == 220, "channel pipeline even squares sum")

	// Defer/panic/recover
	res, err := safeDiv(10, 2)
	check(res == 5 && err == "", "safeDiv normal")
	res, err = safeDiv(10, 0)
	check(res == -1 && err == "division by zero", "safeDiv panic recovery")

	// Map operations
	freq := wordFrequency("the cat sat on the mat the cat")
	check(freq["the"] == 3, "word freq 'the'")
	check(freq["cat"] == 2, "word freq 'cat'")
	check(freq["sat"] == 1, "word freq 'sat'")
	check(freq["mat"] == 1, "word freq 'mat'")

	// Sorted map keys
	keys := make([]string, 0, len(freq))
	for k := range freq {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	check(keys[0] == "cat", "sorted keys first")

	// Fibonacci memo
	fib := makeFibMemo()
	check(fib(10) == 55, "fib(10)")
	check(fib(20) == 6765, "fib(20)")
	check(fib(50) == 12586269025, "fib(50)")

	// String operations
	s := "Hello, Obfuscated World!"
	check(strings.Contains(s, "Obfuscated"), "string contains")
	check(strings.Count(s, "l") == 3, "string count 'l'")
	check(strings.ToUpper(s) == "HELLO, OBFUSCATED WORLD!", "string upper")

	// Goroutine stress: many short-lived goroutines
	var counter int64
	var wg sync.WaitGroup
	for i := 0; i < 1000; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			atomic.AddInt64(&counter, 1)
		}()
	}
	wg.Wait()
	check(counter == 1000, "1000 goroutines counter")

	// Runtime info (sanity)
	check(runtime.NumCPU() > 0, "runtime.NumCPU > 0")
	check(runtime.GOOS == "windows", "runtime.GOOS == windows")
	check(runtime.GOARCH == "amd64", "runtime.GOARCH == amd64")

	fmt.Printf("\n%d passed, %d failed\n", pass, fail)
	if fail > 0 {
		os.Exit(1)
	}
}
