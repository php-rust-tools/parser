package value

type Value interface {
	ToString() string
	ToInt() int

	IsString() bool
	IsInt() bool

	Dump() string
}

func Init() {
	// No-op to avoid unused import errors...
}
