package value

import (
	"fmt"
	"strconv"
)

type Int struct {
	value int
}

func NewInt(value int) *Int {
	return &Int{value}
}

func (i *Int) ToString() string {
	return strconv.Itoa(i.value)
}

func (i *Int) ToInt() int {
	return i.value
}

func (i *Int) IsString() bool {
	return false
}

func (i *Int) IsInt() bool {
	return true
}

func (i *Int) Dump() string {
	return fmt.Sprintf("int(%d)", i.value)
}
