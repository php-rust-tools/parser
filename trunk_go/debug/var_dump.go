package debug

import (
	"fmt"

	"github.com/ryangjchandler/trunk/runtime"
)

func VarDump(args runtime.Args) {
	for _, value := range args.All() {
		fmt.Printf("%s\n", value.Dump())
	}
}
