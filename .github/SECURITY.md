# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability in jsh, please report it responsibly.

### How to Report

**Please do NOT report security vulnerabilities through public GitHub issues.**

Instead, please send an email to: **quinn.josephr@protonmail.com**

Include the following information in your report:

1. **Description**: A clear description of the vulnerability
2. **Impact**: The potential impact of the vulnerability
3. **Steps to Reproduce**: Detailed steps to reproduce the issue
4. **Affected Versions**: Which versions of jsh are affected
5. **Suggested Fix**: If you have a suggested fix, please include it

### What to Expect

- **Acknowledgment**: We will acknowledge receipt of your report within 48 hours
- **Initial Assessment**: We will provide an initial assessment within 7 days
- **Updates**: We will keep you informed about our progress
- **Resolution**: We aim to resolve critical vulnerabilities within 30 days
- **Credit**: We will credit you in the security advisory (unless you prefer to remain anonymous)

### Security Considerations for Shell Software

As a shell interpreter, jsh handles potentially sensitive operations:

- **Command Execution**: jsh executes arbitrary commands on the system
- **Environment Variables**: May contain sensitive information
- **File Operations**: Read/write access to the filesystem
- **Profile Scripts**: Automatic execution of configuration files

### Safe Practices

When using jsh:

1. **Review Scripts**: Always review scripts before running them
2. **Limit Permissions**: Run jsh with appropriate user permissions
3. **Sanitize Input**: Be cautious with user-provided input in scripts
4. **Secure Configuration**: Protect your `.jshrc` and profile files

### Out of Scope

The following are generally not considered vulnerabilities:

- Bugs that require local access with the same privileges as the user
- Issues in third-party dependencies (report to the dependency maintainer)
- Social engineering attacks
- Issues that require physical access to the machine

## Security Updates

Security updates will be released as patch versions (e.g., 0.1.1, 0.1.2).

Subscribe to our releases on GitHub to be notified of security updates.

## Responsible Disclosure

We follow responsible disclosure practices:

1. Vulnerabilities are fixed before public disclosure
2. A security advisory is published on GitHub
3. Credit is given to the reporter (if desired)
4. Users are given time to update before full details are released

Thank you for helping keep jsh and its users safe!

