package keeper

import (
	"github.com/binary-builders/astro/simapp/x/astro/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// InitGenesis initializes the module state from genesis.
func (k Keeper) InitGenesis(ctx sdk.Context, gs types.GenesisState) error {
	// set params if provided
	if gs.Params != nil {
		if err := k.SetParams(ctx, gs.Params); err != nil {
			return err
		}
	}

	// load validators
	for _, v := range gs.Validators {
		if v == nil {
			continue
		}
		if err := k.SetValidator(ctx, v); err != nil {
			return err
		}
	}
	return nil
}

// ExportGenesis exports the current module state to genesis.
func (k Keeper) ExportGenesis(ctx sdk.Context) (*types.GenesisState, error) {
	params, err := k.GetParams(ctx)
	if err != nil {
		// if not found, keep default params in export
		params = types.DefaultParams()
	}

	validators, err := k.GetAllValidators(ctx)
	if err != nil {
		return nil, err
	}

	gs := types.DefaultGenesis()
	gs.Params = params
	gs.Validators = validators
	return gs, nil
}
