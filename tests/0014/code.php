<?php

class Foo2 {
    use B {
        foo as protected bar;
    }
}

class Bar2 { 
    use B,
        C { 
            B::foo insteadof C;
        }
}

class Bar3 {
    use B { B::foo as bar; }
}

class Bar4 {
    use B { foo as bar; }
}
