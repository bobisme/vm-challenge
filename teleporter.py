#!/usr/bin/env python

from functools import cache
import sys
import threading

sys.setrecursionlimit(1_000_000)
threading.stack_size(64 * 1024 * 1024)

MOD = 1 << 15


def recursive_function(r0: int, r1: int, r7: int):
    """
    Literal translation of assembly.

    ```asm
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
    ```
    """
    stack = []

    def main():
        nonlocal r0, r1
        if r0 != 0:
            r0_not_zero()
        else:
            r0 = (r1 + 1) % MOD
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


@cache
def recfn(r0: int, r1: int, r7: int) -> int:
    """
    Non-literal implementation of `recursive_function` with memoization.
    """
    if r0 == 0:
        return (r1 + 1) % MOD
    if r1 == 0:
        return recfn(r0 - 1, r7, r7)
    r1 = recfn(r0, r1 - 1, r7)
    return recfn(r0 - 1, r1, r7)


def ack(r0: int, r1: int, r7: int) -> int:
    """
    Came back after solving to play with the actual Ackermann function.
    Worked out the closed forms while watching stewSquared on stream.
    Got (1, n) from Stew.
    Worked out the (2, n) by hand.
    Found (3, n) using sympy (see `sympy_solve` below()).
    Was able to completely remove memoization.
    """

    match (r0, r1):
        case (0, n):
            out = n + 1
        case (3, 0):
            out = r7 * (r7 + 2) + r7 + 1
        case (m, 0):
            out = ack(m - 1, r7, r7)
        case (1, n):
            out = r7 + n + 1
        case (2, n):
            out = r7 * (n + 2) + n + 1
        case (3, n):
            out = (
                -2 * r7 + ((r7 + 1) ** n) * (r7**3 + 3 * r7**2 + 3 * r7 + 1) - 1
            ) // r7
        case (m, n):
            out = ack(m - 1, ack(m, n - 1, r7), r7)

    return out % MOD


def assert_eq(a, b):
    assert a == b, f"{a} != {b}"


assert_eq(recursive_function(0, 0, 0), recfn(0, 0, 0))
assert_eq(recursive_function(0, 0, 1), recfn(0, 0, 1))
assert_eq(recursive_function(0, 1, 1), recfn(0, 1, 1))
assert_eq(recursive_function(1, 1, 1), recfn(1, 1, 1))
assert_eq(recursive_function(1, 2, 1), recfn(1, 2, 1))
assert_eq(recursive_function(2, 2, 1), recfn(2, 2, 1))

for r0 in range(1, 3):
    for r1 in range(1, 6):
        for r7 in range(1, 10):
            assert_eq(x := recfn(r0, r1, r7), ack(r0, r1, r7))


def brute_force():
    """
    NOTE: this causes a stack overflow.

    ```asm
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
    ```
    """
    for r7 in range(1, (1 << 15) - 1):
        print(f"testing r7 = {r7}")
        # out = recfn(4, 1, r7)
        out = ack(4, 1, r7)
        if out == 6:
            print(f"got {r7=}")
            return r7


def sympy_solve():
    """
    Solving for (3, n) case.
    """
    import sympy

    n = sympy.Symbol("n", integer=True, nonnegative=True)
    r7 = sympy.Symbol("r_7", integer=True, nonnegative=True)
    A = sympy.Function("A")

    # A(3, n) = ack(2, ack(3, n - 1))
    # A(2, n) = r7 * (n + 2) + n + 1
    # A(3, n) = r7 * (ack(3, n-1) + 2) + ack(3, n-1) + 1
    # A(3, n) = (r7 + 1) * ack(3, n-1) + 2*r7  + 1
    # A(3, 0) = r7 * (r7 + 2) + r7 + 1
    # A(3, 0) = r7^2 + 3*r7 + 1
    recurrence = sympy.Eq(A(n), (r7 + 1) * A(n - 1) + 2 * r7 + 1)
    # Case for (3, n=0) => r_7^2 + 3*r_7 + 1
    A_0 = r7**2 + 3 * r7 + 1
    solution = sympy.rsolve(recurrence, A(n), {A(0): A_0})
    assert isinstance(solution, sympy.Expr)
    return sympy.simplify(solution)


if __name__ == "__main__":
    brute_force()
    # t = threading.Thread(target=brute_force)
    # t.start()
    # t.join()
