package main

import (
    "github.com/ryangjchandler/trunk/value"
    "github.com/ryangjchandler/trunk/runtime"
    "github.com/ryangjchandler/trunk/debug"
)

func main() {
    value.Init()
    debug.VarDump(runtime.NewArgs(value.NewInt(1), value.NewInt(2), value.NewInt(3))) 
}