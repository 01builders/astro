package types

// DefaultParams returns the default parameters for the module.
// Relies on the generated Params type from protobufs.
func DefaultParams() *Params { return &Params{} }

// Validate performs basic validation on Params.
func (p *Params) Validate() error { return nil }
