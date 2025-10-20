package types

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
