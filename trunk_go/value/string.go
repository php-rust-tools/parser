package value

import (
	"fmt"
	"strconv"
)

type String struct {
	value string
}

func NewString(value string) *String {
	return &String{value}
}

func (s *String) ToString() string {
	return s.value
}

func (s *String) ToInt() int {
	i, _ := strconv.Atoi(s.value)
	return i
}

func (s *String) IsString() bool {
	return true
}

func (s *String) IsInt() bool {
	return false
}

func (s *String) Dump() string {
	return fmt.Sprintf("string(%d) \"%s\"", len(s.value), s.value)
}
