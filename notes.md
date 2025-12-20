# Notes

## Codes Found

ImoFztWQCvxj // start
BNCyODLfQkIl
pWDWTEfURAdS
rdMkyZhveeIv // found can
JyDQhSbkpyns // use teleporter

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

> A hypothetical such teleportation device would need to have have exactly two
> destinations. One destination would be used when the eighth register is at its
> minimum energy level - this would be the default operation assuming the user
> has no way to control the eighth register. In this situation, the teleporter
> should send the user to a preconfigured safe location as a default.
>
> The second destination, however, is predicted to require a very specific
> energy level in the eighth register. The teleporter must take great care to
> confirm that this energy level is exactly correct before teleporting its user!
> If it is even slightly off, the user would (probably) arrive at the correct
> location, but would experience anomalies in the fabric of reality itself - this
> is, of course, not recommended. Any teleporter would need to test the energy
> level in the eighth register and abort teleportation if it is not exactly
> correct.
>
> This required precision implies that the confirmation mechanism would be very
> computationally expensive. While this would likely not be an issue for large-
> scale teleporters, a hypothetical hand-held teleporter would take billions of
> years to compute the result and confirm that the eighth register is correct.
>
> If you find yourself trapped in an alternate dimension with nothing but a
> hand-held teleporter, you will need to extract the confirmation algorithm,
> reimplement it on more powerful hardware, and optimize it. This should, at the
> very least, allow you to determine the value of the eighth register which would
> have been accepted by the teleporter's confirmation mechanism.
>
> Then, set the eighth register to this value, activate the teleporter, and
> bypass the confirmation mechanism. If the eighth register is set correctly, no
> anomalies should be experienced, but beware - if it is set incorrectly, the
> now-bypassed confirmation mechanism will not protect you!

```asm
/* 0x1561 */    jf      r7,     0x15fb         ; if r7 == 0, jump to safe room
/* 0x1564 */    push    r0                     ; otherwise make a function call
/* 0x1566 */    push    r1
/* 0x1568 */    push    r2
/* 0x156a */    set     r0,     0x70ba
/* 0x156d */    set     r1,     0x0611
/* 0x1570 */    add     r2,     0x3fe5, 0x0003
/* 0x1574 */    call    0x05c8                 ; call 0x05c8
```

```asm
/* 0x05c8 */    push    r0
/* 0x05ca */    push    r3
/* 0x05cc */    push    r4
/* 0x05ce */    push    r5
/* 0x05d0 */    push    r6
/* 0x05d2 */    set     r6,     r0
/* 0x05d5 */    set     r5,     r1
/* 0x05d8 */    rmem    r4,     r0
/* 0x05db */    set     r1,     0x0000
/* 0x05de */    add     r3,     0x0001, r1
/* 0x05e2 */    gt      r0,     r3,     r4
/* 0x05e6 */    jt      r0,     0x05f9
/* 0x05e9 */    add     r3,     r3,     r6
/* 0x05ed */    rmem    r0,     r3
/* 0x05f0 */    call    r5                      ; 0x0611
/* 0x05f2 */    add     r1,     r1,     0x0001
/* 0x05f6 */    jt      r1,     0x05de
/* 0x05f9 */    pop     r6
/* 0x05fb */    pop     r5
/* 0x05fd */    pop     r4
/* 0x05ff */    pop     r3
/* 0x0601 */    pop     r0
/* 0x0603 */    ret
/* 0x0604 */    push    r1
/* 0x0606 */    set     r1,     0x060e
/* 0x0609 */    call    0x05c8
```

```asm
/* 0x0611 */    push    r1
/* 0x0613 */    set     r1,     r2
/* 0x0616 */    call    0x0863
```

```asm
/* 0x0863 */    push    r1
/* 0x0865 */    push    r2
/* 0x0867 */    and     r2,     r0,     r1
/* 0x086b */    not     r2,     r2
/* 0x086e */    or      r0,     r0,     r1
/* 0x0872 */    and     r0,     r0,     r2
/* 0x0876 */    pop     r2
/* 0x0878 */    pop     r1
/* 0x087a */    ret
```

```asm
/* 0x0618 */    out     r0
/* 0x061a */    pop     r1
/* 0x061c */    ret
```

```asm
/* 0x1576 */    pop     r2
/* 0x1578 */    pop     r1
/* 0x157a */    pop     r0
/* 0x157c */    noop
/* 0x157d */    noop
/* 0x157e */    noop
/* 0x157f */    noop
/* 0x1580 */    noop
/* 0x1581 */    set     r0,     0x0004
/* 0x1584 */    set     r1,     0x0001
/* 0x1587 */    call    0x17a1
```

```asm
/* 0x17a1 */    jt      r0,     0x17a9
/* 0x17a4 */    add     r0,     r1,     0x0001
/* 0x17a8 */    ret
```

```asm
/* 0x17a9 */    jt      r1,     0x17b6
/* 0x17ac */    add     r0,     r0,     0x7fff
/* 0x17b0 */    set     r1,     r7
/* 0x17b3 */    call    0x17a1
/* 0x17b5 */    ret
```

```asm
/* 0x17b6 */    push    r0
/* 0x17b8 */    add     r1,     r1,     0x7fff
/* 0x17bc */    call    0x17a1
/* 0x17be */    set     r1,     r0
/* 0x17c1 */    pop     r0
/* 0x17c3 */    add     r0,     r0,     0x7fff
/* 0x17c7 */    call    0x17a1
/* 0x17c9 */    ret
```
