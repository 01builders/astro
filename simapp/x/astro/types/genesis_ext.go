package types

// DefaultGenesis returns the default genesis state.
func DefaultGenesis() *GenesisState {
	return &GenesisState{
		Params: DefaultParams(),
	}
}

// Validate validates the genesis state.
func (gs *GenesisState) Validate() error {
	return gs.Params.Validate()
}
