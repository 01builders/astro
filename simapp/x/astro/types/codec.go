package types

import (
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
)

// RegisterLegacyAminoCodec registers the necessary x/astro interfaces and concrete types
// on the provided LegacyAmino codec. This is a no-op for now.
func RegisterLegacyAminoCodec(_ *codec.LegacyAmino) {}

// RegisterInterfaces registers interfaces and implementations of the x/astro module.
// This is a no-op for now.
func RegisterInterfaces(_ codectypes.InterfaceRegistry) {}
