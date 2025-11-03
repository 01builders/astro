package types

// DefaultGenesis returns the default genesis state.
func DefaultGenesis() *GenesisState {
	p := DefaultParams()
	return &GenesisState{Params: &p}
}

// Validate validates the genesis state.
func (gs *GenesisState) Validate() error {
	return gs.Params.Validate()
}
