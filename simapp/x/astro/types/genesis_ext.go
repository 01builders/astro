package types

import "fmt"

// DefaultGenesis returns the default genesis state.
func DefaultGenesis() *GenesisState {
	p := DefaultParams()
	return &GenesisState{Params: &p}
}

// Validate validates the genesis state.
func (gs *GenesisState) Validate() error {
	if err := gs.Params.Validate(); err != nil {
		return err
	}
	// validate validators list
	seen := make(map[string]struct{})
	for _, v := range gs.Validators {
		if v == nil {
			continue
		}
		if v.Moniker == "" {
			return fmt.Errorf("validator moniker cannot be empty")
		}
		if len(v.Moniker) > MaxMonikerLength {
			return fmt.Errorf("validator moniker exceeds maximum length (%d)", MaxMonikerLength)
		}
		if _, ok := seen[v.Moniker]; ok {
			return fmt.Errorf("duplicate validator moniker: %s", v.Moniker)
		}
		seen[v.Moniker] = struct{}{}
		if err := ValidateIdentity(v.Identity); err != nil {
			return fmt.Errorf("invalid validator identity: %w", err)
		}
	}
	return nil
}
