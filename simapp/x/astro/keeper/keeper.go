package keeper

import (
	"context"
	"errors"
	"fmt"

	"cosmossdk.io/collections"
	"cosmossdk.io/core/store"
	"github.com/binary-builders/astro/simapp/x/astro/types"
	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/proto/tendermint/crypto"
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
	Params collections.Item[types.Params]
	// Validators keyed by moniker
	Validators collections.Map[string, types.Validator]
}

// NewKeeper returns a new Keeper instance.
func NewKeeper(cdc codec.Codec, storeService store.KVStoreService, authority string) Keeper {
	sb := collections.NewSchemaBuilder(storeService)

	k := Keeper{
		cdc:          cdc,
		storeService: storeService,
		authority:    authority,
		Params:       collections.NewItem(sb, types.ParamsKey, "params", codec.CollValue[types.Params](cdc)),
		Validators:   collections.NewMap(sb, types.ValidatorsPrefix, "validators", collections.StringKey, codec.CollValue[types.Validator](cdc)),
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
func (k Keeper) SetParams(ctx sdk.Context, params types.Params) error {
	return k.Params.Set(ctx.Context(), params)
}

// GetParams retrieves the module parameters.
func (k Keeper) GetParams(ctx sdk.Context) (types.Params, error) {
	return k.Params.Get(ctx.Context())
}

// SetValidator stores or updates a validator by its moniker.
func (k Keeper) SetValidator(ctx sdk.Context, v types.Validator) error {
	// basic validation
	if v.Moniker == "" {
		return fmt.Errorf("validator moniker cannot be empty")
	}
	if len(v.Moniker) > types.MaxMonikerLength {
		return fmt.Errorf("validator moniker exceeds maximum length (%d)", types.MaxMonikerLength)
	}
	if err := types.ValidateIdentity(v.Identity); err != nil {
		return fmt.Errorf("invalid validator identity: %w", err)
	}
	if len(v.Identity) > 0 {
		dup := false
		err := k.Validators.Walk(ctx.Context(), nil, func(m string, existing types.Validator) (bool, error) {
			if m != v.Moniker && len(existing.Identity) > 0 && string(existing.Identity) == string(v.Identity) {
				dup = true
				return true, nil
			}
			return false, nil
		})
		if err != nil {
			return err
		}
		if dup {
			return fmt.Errorf("duplicate validator identity")
		}
	}
	return k.Validators.Set(ctx.Context(), v.Moniker, v)
}

// GetValidator returns a validator by moniker.
// The boolean indicates whether a value was found.
func (k Keeper) GetValidator(ctx sdk.Context, moniker string) (types.Validator, bool, error) {
	v, err := k.Validators.Get(ctx.Context(), moniker)
	if err != nil {
		if errors.Is(err, collections.ErrNotFound) {
			return types.Validator{}, false, nil
		}
		return types.Validator{}, false, err
	}
	return v, true, nil
}

// GetAllValidators collects and returns all validators.
func (k Keeper) GetAllValidators(ctx sdk.Context) ([]types.Validator, error) {
	var vals []types.Validator
	if err := k.Validators.Walk(ctx.Context(), nil, func(_ string, v types.Validator) (bool, error) {
		vals = append(vals, v)
		return false, nil
	}); err != nil {
		return nil, err
	}
	return vals, nil
}

// ApplyAndReturnValidatorSetUpdates implements the minimal interface expected by genutil.
// No updates are applied here; Astro manages consensus identity externally.
func (k Keeper) ApplyAndReturnValidatorSetUpdates(ctx context.Context) ([]abci.ValidatorUpdate, error) {
	// Note: We don't modify Tendermint's validator set here; we merely surface
	// the current number of Astro validators as updates with constant voting power
	// the private key is represented as Ed25519
	// genutil interface expectations without applying staking logic.
	updates := make([]abci.ValidatorUpdate, 0)
	err := k.Validators.Walk(ctx, nil, func(_ string, val types.Validator) (bool, error) {
		updates = append(updates, abci.ValidatorUpdate{
			PubKey: crypto.PublicKey{
				Sum: &crypto.PublicKey_Ed25519{
					Ed25519: val.Identity,
				},
			},
			Power: 1,
		})
		return false, nil
	})
	if err != nil {
		return nil, err
	}
	return updates, nil
}
