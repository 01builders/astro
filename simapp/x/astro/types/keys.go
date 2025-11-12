package types

import "cosmossdk.io/collections"

const (
	// ModuleName defines the module name
	ModuleName = "astro"

	// StoreKey is the string store representation
	StoreKey = ModuleName

	// RouterKey is the message route for slashing
	RouterKey = ModuleName

	// MemStoreKey is the mem store key
	MemStoreKey = "mem_" + ModuleName
)

// Collections prefixes for state storage
var (
    // ParamsKey is the singleton key for module params
    ParamsKey = collections.NewPrefix("AstroParams")

    // ValidatorsPrefix stores validators by their moniker
    ValidatorsPrefix = collections.NewPrefix("AstroValidators")
)
