---
type: Practice
title: Safe C Functions — Banned Memory and String Operations
description: Replace unbounded C memory/string functions (memcpy, strcpy, sprintf) with bounded _s variants or snprintf/fgets, and enable compiler hardening flags to catch overflows at build time.
tags: [devsecops, security, c, cpp, memory-safety, buffer-overflow, compiler-flags]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-17"
  last-used: "2026-07-17"
sources:
  - id: codeguard-1-safe-c-functions
    resource: ".devin/rules/codeguard-1-safe-c-functions.md"
    title: "job-aide"
  - id: codeguard-0-devops-ci-cd-containers
    resource: ".devin/rules/codeguard-0-devops-ci-cd-containers.md"
    title: "job-aide (C/C++ toolchain hardening section)"
---


# Safe C Functions — Banned Memory and String Operations

## Failure Mode

C and C++ code that uses unbounded memory and string functions is the classic
source of buffer overflow vulnerabilities. Functions like `strcpy`, `sprintf`,
and `gets` perform no bounds checking — they copy or format bytes until a null
terminator is reached or the format string is exhausted, writing past the
destination buffer if the input is larger than expected. This is the most
common pattern behind exploitable memory corruption bugs.

## Banned Memory Functions

These functions perform no boundary checking on input parameters and must
never be used:

| Banned | Safe Replacement |
|--------|-----------------|
| `memcpy()` | `memcpy_s()` |
| `memset()` | `memset_s()` |
| `memmove()` | `memmove_s()` |
| `memcmp()` | `memcmp_s()` |
| `bzero()` | `memset_s()` |
| `memzero()` | `memset_s()` |

### Safe Memory Copy Pattern

```c
// Bad — no boundary checking
memcpy(dest, src, size);

// Good — bounded with error handling
errno_t result = memcpy_s(dest, dest_max_size, src, size);
if (result != 0) {
    // Handle error: src too long or invalid parameters
    return ERROR;
}
```

## Banned String Functions

These functions can cause buffer overflows and must never be used:

| Banned | Safe Replacement |
|--------|-----------------|
| `gets()` | `fgets()` |
| `strcpy()` | `strcpy_s()` or `snprintf()` |
| `strcat()` | `strcat_s()` or `snprintf()` |
| `sprintf()` / `vsprintf()` | `snprintf()` |
| `strcmp()` | `strcmp_s()` |
| `strlen()` | `strnlen_s()` |
| `strstr()` | `strstr_s()` |
| `strtok()` | `strtok_s()` (C11 Annex K) or `strtok_r()` (POSIX) |

### Safe String Copy Pattern

```c
// Bad — buffer overflow risk
char dest[256];
strcpy(dest, src);

// Good — bounded with error handling
char dest[256];
errno_t result = strcpy_s(dest, sizeof(dest), src);
if (result != 0) {
    // Handle error: src too long or invalid parameters
    return ERROR;
}
```

### Safe Formatted String Pattern

```c
// Bad — no bounds on output buffer
sprintf(dest, "%s", source_string);

// Good — bounded
snprintf(dest, sizeof(dest), "%s", source_string);
```

## scanf Family

The `scanf` family is medium risk — `%s` without a width limit causes buffer
overflows. Either use width specifiers or read with `fgets` then parse with
`sscanf`:

```c
// Bad — no width limit
scanf("%s", user_name);

// Good — width specifier
scanf("%127s", user_name);

// Better — read line then parse
if (fgets(user_name, sizeof(user_name), stdin)) {
    user_name[strcspn(user_name, "\n")] = 0;
}
```

## strncpy Pitfall

`strncpy` is a common but imperfect replacement — it does not guarantee
null-termination if the source is as long as the destination buffer. Always
enforce explicit termination:

```c
char dest[10];
strncpy(dest, source, sizeof(dest) - 1);
dest[sizeof(dest) - 1] = '\0';
```

## Compiler Hardening Flags

Enable these protective compiler flags to catch buffer overflows at compile
time and runtime:

| Flag | Purpose |
|------|---------|
| `-fstack-protector-all` / `-fstack-protector-strong` | Detect stack buffer overflows via canaries |
| `-fsanitize=address` | AddressSanitizer — catch memory errors during development |
| `-D_FORTIFY_SOURCE=2` | Runtime bounds checking for `strcpy`, `strcat`, `sprintf`, etc. |
| `-Wformat -Wformat-security` | Catch format string vulnerabilities |
| `-Wall -Wextra -Wconversion` | Enable additional warnings |
| `-fPIE` / `-pie` | Position-independent executables for ASLR |
| `-fsanitize=cfi` (with LTO) | Control flow integrity |

Additional linker hardening: RELRO/now, noexecstack, NX/DEP, and ASLR. Verify
flags in CI with `checksec` and fail builds if protections are missing.

## Common Pitfalls

1. **Wrong size parameter**: Use destination buffer size, not source length.
   `strcpy_s(dest, sizeof(dest), src)` — not `strcpy_s(dest, strlen(src), src)`.
2. **Ignoring return values**: Always check `errno_t` return values from `_s`
   functions.
3. **sizeof on pointers**: `sizeof(char*)` is 8, not the buffer size. Pass
   buffer size as an explicit parameter when the buffer is accessed via pointer.

## Code Review Checklist

- [ ] No unsafe memory functions (`memcpy`, `memset`, `memmove`, `memcmp`, `bzero`)
- [ ] No unsafe string functions (`strcpy`, `strcat`, `sprintf`, `gets`, `strtok`)
- [ ] All memory operations use `*_s()` variants with proper size parameters
- [ ] Buffer sizes calculated using `sizeof()` or known limits
- [ ] All `errno_t` return values handled
- [ ] Strings properly null-terminated
- [ ] Compiler hardening flags enabled and verified in CI
