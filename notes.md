# Notes

## Codes Found

LDOb7UGhTi // arch-spec
ImoFztWQCvxj // start
BNCyODLfQkIl // self test
pWDWTEfURAdS // write on tablet
rdMkyZhveeIv // found can
JyDQhSbkpyns // use teleporter first time
NBlOWKLbTMgY // teleport to beach
qo8HqHOwU8Wi // look in the mirror

~~JhxmqOXvzQQM~~ // teleport to beach NOT VALID!?

## Puzzles

### Coins

_+_ \* _^2 +_^3 - \_ = 399

blue coin has 9 dots
red coin has 2 dots
shiny coin has a pentagon
concave coin has 7 dots
corroded coin has a triangle

9 + 2 \* 5**2 + 7**3 - 3 = 399

### Teleporter

```python
def recursive_function(r0: int, r1: int, r7: int):
    """
    Literal translation of assembly.

    [0x17a1] jt     r0, 0x17a9            ; if r0 != 0, jump to r0_not_zero
    [0x17a4] add    r0, r1, 0x0001        ; r0 = r1 + 1
    [0x17a8] ret
    r0_not_zero:
    [0x17a9] jt     r1, 0x17b6            ; if r1 != 0, jump to r1_not_zero
    [0x17ac] add    r0, r0, 0x7fff        ; r0 -= 1
    [0x17b0] set    r1, r7                ; r1 = r7
    [0x17b3] call   0x17a1                ; recursive_func(r0, r1), aka recursive_func(r0 - 1, r7)
    [0x17b5] ret
    r1_not_zero:
    [0x17b6] push   r0                    ; r0 -> stack
    [0x17b8] add    r1, r1, 0x7fff        ; r1 -= 1
    [0x17bc] call   0x17a1                ; recursive_func(r0, r1), aka recursive_func(r0, r1 - 1)
    [0x17be] set    r1, r0                ; r1 = r0
    [0x17c1] pop    r0                    ; r0 <- stack
    [0x17c3] add    r0, r0, 0x7fff        ; r0 -= 1
    [0x17c7] call   0x17a1                ; recursive_func(r0, r1), aka recursive_func(r0 -1, r1)
    [0x17c9] ret
    """
    stack = []

    def main():
        nonlocal r0, r1
        if r0 != 0:
            r0_not_zero()
        else:
            r0 = (r1 + 1) % MAX_U15
            return r0

    def r0_not_zero():
        nonlocal r0, r1
        if r1 != 0:
            r1_not_zero()
        else:
            r0 -= 1
            r1 = r7
            return main()

    def r1_not_zero():
        nonlocal r0, r1
        stack.append(r0)
        r1 -= 1
        main()
        r1 = r0
        r0 = stack.pop()
        r0 -= 1
        return main()

    main()
    return r0
```

```python
def brute_force():
    """
    NOTE: this causes a stack overflow.

    [0x1581] set    r0, 0x0004
    [0x1584] set    r1, 0x0001
    [0x1587] call   0x17a1                ; recursive_func(4, 1)
    [0x1589] eq     r1, r0, 0x0006        ; if r0 != 6:
    [0x158d] jf     r1, 0x15e1            ;   jump to 0x15e1
    [0x1590] push   r0                    ; else:
    [0x1592] push   r1                    ;   #
    [0x1594] push   r2                    ;   #
    [0x1596] set    r0, 0x7163            ;   r0 = 0x7163
    [0x1599] set    r1, 0x0611            ;   r1 = 0x0611
    [0x159c] add    r2, 0x1f73, 0x3a78    ;   r2 = 0x59eb
    [0x15a0] call   0x05c8                ;   0x05c8(r0, r1, r2)
    """
    for r7 in range(1, (1 << 15) - 1):
        print(f"testing r7 = {r7}")
        out = recfn(4, 1, r7)
        if out == 6:
            return r7
```

See ./teleporter.py for details. It wouldn't run, so it was rewritten in ./src/main.rs.

FOUND r7 = 25734 // originally had 9946 because I was modding by 2^16 - 1 instead of 2^16. Found after checking all the codes.

Needed to set r7 and bypass the call to the recursive function to teleport correctly.

### The orb

```
 *  8  -  1 30 & hourglass
 4  * 11  *
 +  4  - 18
22  -  9  *
```

See ./orb.py for solution.

iW8UwOHpH8op // not valid... but it's in a mirror
po8HpHOwU8Wi // is the reverse, but it's not valid either...
qo8HqHOwU8Wi // if I flip the p's to q's WORKS
