package keeper

import (
	"github.com/binary-builders/astro/simapp/x/astro/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// InitGenesis initializes the module state from genesis.
func (k Keeper) InitGenesis(ctx sdk.Context, gs types.GenesisState) error {
	// initialize module state here as needed
	_ = ctx
	_ = gs
	return nil
}

// ExportGenesis exports the current module state to genesis.
func (k Keeper) ExportGenesis(ctx sdk.Context) (*types.GenesisState, error) {
	_ = ctx
	// populate from state when implemented
	return types.DefaultGenesis(), nil
}
