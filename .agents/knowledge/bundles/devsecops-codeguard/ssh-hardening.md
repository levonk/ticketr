---
type: Practice
title: SSH Hardening — PermitRootLogin, Key Types, and fail2ban
description: Disable root password login, use ed25519-only host keys, enforce PasswordAuthentication no, and run fail2ban to reduce brute-force risk on cloud servers.
tags: [devsecops, security, ssh, hardening, fail2ban, ed25519]
date:
  created: "2026-07-18"
  knowledge-basis: "2026-07-17"
  last-used: "2026-07-17"
sources:
  - id: infrahub-final-audit-yml
    resource: "https://github.com/levonk/infrahub/blob/main/shared/active/02-config/ansible/playbooks/final-audit.yml"
    title: "infrahub final-audit.yml"
  - id: infrahub-agents-md-security-audit-guidelines
    resource: "https://github.com/levonk/infrahub/blob/main/AGENTS.md"
    title: "infrahub AGENTS.md Security Audit Guidelines"
---


# SSH Hardening

## Failure Mode

Cloud servers are compromised through weak SSH configuration: root login with
password, RSA host keys, or no brute-force protection.

## Symptoms

- `PermitRootLogin yes` is set in `/etc/ssh/sshd_config`.
- Password authentication is enabled.
- RSA, ECDSA, or DSA host keys are present alongside ed25519.
- `fail2ban` is not installed or not running.
- SSH logs show repeated password attempts from internet scanners.

## Practice

### PermitRootLogin

- Set `PermitRootLogin no` for maximum security.
- `PermitRootLogin prohibit-password` is acceptable when root login with SSH
  keys is required (e.g., some Oracle Cloud templates).

### Authentication

- `PasswordAuthentication no` — require keys only.
- `ChallengeResponseAuthentication no` (legacy) or `KbdInteractiveAuthentication no`.
- `PubkeyAuthentication yes`.

### Host Keys

- Use ed25519 host keys only.
- Remove or disable RSA, ECDSA, and DSA host keys.
- Update `HostKey` directives in `/etc/ssh/sshd_config`:

```
HostKey /etc/ssh/ssh_host_ed25519_key
```

### fail2ban

- Install and enable `fail2ban`.
- Configure an SSH jail with a short `maxretry` and reasonable `bantime`.
- Verify status with `fail2ban-client status sshd`.

### Ansible Automation

The infrahub `final-audit.yml` playbook validates these settings as part of the
security audit. Use Ansible `lineinfile` or a dedicated `sshd_config` template
for enforcement.
