package keeper

import (
	storetypes "cosmossdk.io/store/types"
	"github.com/cosmos/cosmos-sdk/codec"
)

// Keeper provides state management for the x/astro module.
type Keeper struct {
	cdc          codec.Codec
	storeService storetypes.KVStoreService
	authority    string
}

// NewKeeper returns a new Keeper instance.
func NewKeeper(cdc codec.Codec, storeService storetypes.KVStoreService, authority string) Keeper {
	return Keeper{
		cdc:          cdc,
		storeService: storeService,
		authority:    authority,
	}
}

// Authority returns the module authority address (typically the gov module account).
func (k Keeper) Authority() string { return k.authority }
