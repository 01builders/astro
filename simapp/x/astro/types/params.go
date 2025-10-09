package types

// Params defines the parameters for the x/astro module.
// Add real parameters as needed.
type Params struct{}

// DefaultParams returns the default parameters for the module.
func DefaultParams() Params { return Params{} }

// Validate performs basic validation on Params.
func (p Params) Validate() error { return nil }
