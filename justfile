# Try the pick popup in a throwaway herdr; exit or ctrl+x q leaves. `just preview --leaders --stripes` turns [picker] switches on
preview *switches:
    scripts/preview-popup {{switches}}

# Fail when the code breaks the standards the build enforces, then prove the check itself
standards:
    scripts/check-standards
    scripts/check-standards.test
