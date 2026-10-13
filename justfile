# Try the pick popup in a throwaway herdr; exit or ctrl+x q leaves. `just preview "stripes = true"` adds a [pick] line
preview pick="":
    PREVIEW_PICK="{{pick}}" scripts/preview-popup
