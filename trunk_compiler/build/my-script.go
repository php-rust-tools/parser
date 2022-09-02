package main

import (
    "github.com/ryangjchandler/trunk/value"
    "github.com/ryangjchandler/trunk/runtime"
    "github.com/ryangjchandler/trunk/constructs"
    "github.com/ryangjchandler/trunk/math"
)

func main() {
    value.Init()
    constructs.Echo(math.Rand(runtime.NewArgs(value.NewInt(0), value.NewInt(5)))) 
}