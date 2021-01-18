TODO:

- Checking f32's as integers when integers required.
  Need to check mantisa.
- Implement Networks, overhaul devices to be owned by networks instead of IC's,
  and have IC's reference devices on networks.
- **Idea:**
  - Recognize jump functions, and able to decompose them.

e.g.:
```mips
main:
jal f
j main

f:
move r0 3.14
```
should become
```mips
main:
move r0 3.14
j main
```

