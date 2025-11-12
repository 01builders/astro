package types

// Validation constants for the astro module.
const (
	// MaxMonikerLength sets an upper bound on validator moniker length.
	MaxMonikerLength           = 64
	BLS12381MinSigPubKeySizeG1 = 48
	BLS12381MinSigPubKeySizeG2 = 96
)

// ValidateIdentity is a hook that can be set by upstream/commonware to enforce
// specific validation on the BLS MinSig identity bytes (size/format).
// By default it performs no checks.
var ValidateIdentity = func(_ []byte) error { return nil }
