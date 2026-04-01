# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |
| < 0.1.0 | :x:                |

## Reporting a Vulnerability

**Please do not create public issues for security vulnerabilities.**

### How to Report

1. **Create a draft security advisory** using GitHub's Security Advisories feature, OR
2. **Email directly** to: [INSERT SECURITY EMAIL]

### What to Include

- **Description** of the vulnerability
- **Steps to reproduce** the issue
- **Potential impact** of the vulnerability
- **Suggested fix** (if you have one)
- **Your contact information** for follow-up

### Response Timeline

- **Acknowledgment**: Within 48 hours
- **Initial assessment**: Within 1 week
- **Fix development**: Based on severity
- **Public disclosure**: After fix is available

## Security Best Practices

### For Users

- **Keep updated** - Use the latest version
- **Verify downloads** - Check checksums when available
- **Report issues** - Follow responsible disclosure

### For Contributors

- **Never commit secrets** - API keys, passwords, tokens
- **Validate input** - All user input must be validated
- **Use secure dependencies** - Keep dependencies updated
- **Follow coding standards** - Security-focused development

## Known Security Features

### Application Security

- ✅ **Content Security Policy (CSP)** enabled
- ✅ **CSV injection protection** implemented
- ✅ **Input validation** on all user inputs
- ✅ **Secure credential storage** (OS keychain)
- ✅ **Error message sanitization**

### Development Security

- ✅ **Dependency scanning** (cargo audit, npm audit)
- ✅ **Code review** required for all PRs
- ✅ **Automated testing** for security features
- ✅ **Security-focused linting**

## Security Updates

Security updates are released as patch versions (e.g., 0.1.1, 0.1.2).

### Notification

Users will be notified of security updates through:
- GitHub Releases
- Security advisories
- Project announcements

## Bug Bounty Program

Currently, we do not have a formal bug bounty program. However, we greatly appreciate responsible disclosure and will acknowledge contributors who help improve our security.

## Contact

For security-related questions or concerns:
- **Email**: [INSERT EMAIL]
- **GitHub Security Advisories**: [Link to advisories]

---

Thank you for helping keep Link Mage secure! 🔒
