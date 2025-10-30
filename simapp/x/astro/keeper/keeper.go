package keeper

import (
	"errors"
	"fmt"

	"cosmossdk.io/collections"
	"cosmossdk.io/core/store"
	"github.com/binary-builders/astro/simapp/x/astro/types"
	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// Keeper provides state management for the x/astro module.
type Keeper struct {
	cdc          codec.Codec
	storeService store.KVStoreService
	authority    string

	// collections schema and stores
	schema collections.Schema
	Params collections.Item[*types.Params]
	// Validators keyed by moniker
	Validators collections.Map[string, *types.Validator]
}

// NewKeeper returns a new Keeper instance.
func NewKeeper(cdc codec.Codec, storeService store.KVStoreService, authority string) Keeper {
	sb := collections.NewSchemaBuilder(storeService)

	k := Keeper{
		cdc:          cdc,
		storeService: storeService,
		authority:    authority,
		Params:       collections.NewItem(sb, types.ParamsKey, "params", codec.CollValue[*types.Params](cdc)),
		Validators:   collections.NewMap(sb, types.ValidatorsPrefix, "validators", collections.StringKey, codec.CollValue[*types.Validator](cdc)),
	}

	schema, err := sb.Build()
	if err != nil {
		// In keeper construction failing to build the schema is unrecoverable
		panic(err)
	}
	k.schema = schema
	return k
}

// Authority returns the module authority address (typically the gov module account).
func (k Keeper) Authority() string { return k.authority }

// SetParams stores the module parameters.
func (k Keeper) SetParams(ctx sdk.Context, params *types.Params) error {
	return k.Params.Set(ctx, params)
}

// GetParams retrieves the module parameters.
func (k Keeper) GetParams(ctx sdk.Context) (*types.Params, error) {
	return k.Params.Get(ctx)
}

// SetValidator stores or updates a validator by its moniker.
func (k Keeper) SetValidator(ctx sdk.Context, v *types.Validator) error {
	// basic validation
	if v.Moniker == "" {
		return fmt.Errorf("validator moniker cannot be empty")
	}
	return k.Validators.Set(ctx, v.Moniker, v)
}

// GetValidator returns a validator by moniker.
// The boolean indicates whether a value was found.
func (k Keeper) GetValidator(ctx sdk.Context, moniker string) (*types.Validator, bool, error) {
	v, err := k.Validators.Get(ctx, moniker)
	if err != nil {
		if errors.Is(err, collections.ErrNotFound) {
			return nil, false, nil
		}
		return nil, false, err
	}
	return v, true, nil
}

// GetAllValidators collects and returns all validators.
func (k Keeper) GetAllValidators(ctx sdk.Context) ([]*types.Validator, error) {
	var vals []*types.Validator
	if err := k.Validators.Walk(ctx, nil, func(_ string, v *types.Validator) (bool, error) {
		vals = append(vals, v)
		return false, nil
	}); err != nil {
		return nil, err
	}
	return vals, nil
}
