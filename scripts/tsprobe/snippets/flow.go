// Package flow is a probe sample (analysis-track §5.3, round 3): every
// statement, declaration and access shape the flow/1 lowering table
// names.
package flow

import (
	"fmt"
	"log"
	"os"
	"runtime"
)

type point struct{ x, y int }

type counter struct{ n int }

func (c *counter) bump(by int) int {
	c.n += by
	return c.n
}

func branches(a int, b, c int, rest ...int) (label string, total int) {
	if a > 0 {
		label = "pos"
	} else if a < 0 {
		label = "neg"
	} else {
		label = "zero"
	}
	if v, ok := lookup(a); ok {
		total = v
	}
	for i := 0; i < b; i++ {
		if i == c {
			continue
		}
		if i > c {
			break
		}
		total += i
	}
	for _, v := range rest {
		total -= v
	}
	for key := range rest {
		total += key
	}
	n := 3
	for n > 0 {
		n--
	}
	for {
		if total > 10 {
			break
		}
		total = total + 1
	}
outer:
	for i := 0; i < b; i++ {
		for j := 0; j < c; j++ {
			if i == j {
				continue outer
			}
			if i > j {
				break outer
			}
		}
	}
	if total < 0 {
		goto done
	}
	total++
done:
	return label, total
}

func lookup(k int) (int, bool) {
	const limit = 3
	return k, k > limit
}

func switching(kind int, x interface{}, ch chan int, quit chan bool) int {
	score := 0
	switch k := kind * 2; k {
	case 1, 2:
		score = 1
		fallthrough
	case 3:
		score += 2
	default:
		score = -1
	}
	switch {
	case score > 5:
		score = 5
	}
	switch y := x; v := y.(type) {
	case int:
		score += v
	case string:
		score += len(v)
	default:
		score = 0
	}
	select {
	case got := <-ch:
		score = got
	case quit <- true:
		score = 0
	}
	return score
}

func accesses(p *point, items []int, m map[string]int, flag bool) int {
	x := 0
	x = x + 1
	x += p.x
	p.y = x
	items[0] = x
	m["k"] = x
	ptr := &x
	*ptr = 5
	x, y := 7, 8
	var z int
	var w, u = 1, 2
	var (
		p2 = 1
		q2 = 2
	)
	if flag && lookupOK(&z) {
		z = 1
	}
	if flag || lookupOK(&w) {
		w = 3
	}
	_ = y
	_, err := fmt.Println(x, z, w, u, p2, q2)
	if err != nil {
		return 0
	}
	shadow := 1
	{
		shadow := 2
		shadow++
		fmt.Println(shadow)
	}
	built := point{x: shadow, y: x}
	return built.x
}

func lookupOK(p *int) bool {
	*p = 3
	return true
}

func closures(values []int) func() int {
	base := 10
	scale := 2
	add := func(d int) int {
		scale += d
		return d + base
	}
	defer fmt.Println(base)
	go func() {
		fmt.Println(scale)
	}()
	return func() int { return add(values[0]) }
}

func fatal(code int) {
	if code > 0 {
		os.Exit(code)
	}
	if code < 0 {
		log.Fatalf("negative %d", code)
	}
	if code == 0 {
		panic("zero")
	}
	runtime.Goexit()
}

func blockForever() {
	select {}
}
