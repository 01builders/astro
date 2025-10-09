package types

import (
	errorsmod "cosmossdk.io/errors"
)

// x/astro module sentinel errors
var (
	ErrSample = errorsmod.Register(ModuleName, 1, "sample error")
)
