package math

import (
	"math/rand"
	"time"

	"github.com/ryangjchandler/trunk/runtime"
	"github.com/ryangjchandler/trunk/value"
)

func Rand(args runtime.Args) value.Value {
	if args.IsEmpty() {
		return value.NewInt(rand.Int())
	}

	if args.Count() < 2 {
		panic("todo: add in error here")
	}

	min := args.At(0)
	max := args.At(1)

	if !min.IsInt() {
		panic("rand(): argument 1 ($min) must be of type int")
	}

	if !max.IsInt() {
		panic("rand(): argument 2 ($max) must be of type int")
	}

	min_i := min.ToInt()
	max_i := max.ToInt()

	return value.NewInt(rand.Intn(max_i-min_i) + min_i)
}

func init() {
	rand.Seed(time.Now().UnixNano())
}
