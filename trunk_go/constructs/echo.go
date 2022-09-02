package constructs

import (
	"fmt"

	"github.com/ryangjchandler/trunk/value"
)

func Echo(values ...value.Value) {
	for _, value := range values {
		fmt.Print(value.ToString())
	}
}
