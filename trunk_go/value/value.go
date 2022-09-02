package value

type Value interface {
	ToString() string
	ToInt() int

	IsString() bool
	IsInt() bool
}

func Init() {
	// No-op to avoid unused import errors...
}
