package runtime

import "github.com/ryangjchandler/trunk/value"

type Args struct {
	values []value.Value
}

func NewArgs(values ...value.Value) Args {
	return Args{values}
}

func (args *Args) At(index int) value.Value {
	return args.values[index]
}

func (args *Args) IsEmpty() bool {
	return len(args.values) == 0
}

func (args *Args) Count() int {
	return len(args.values)
}

func (args *Args) All() []value.Value {
	return args.values
}
