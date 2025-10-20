package astro

import (
	"context"

	"cosmossdk.io/core/address"
	"cosmossdk.io/math"
	abci "github.com/cometbft/cometbft/abci/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	stakingtypes "github.com/cosmos/cosmos-sdk/x/staking/types"
)

type Keeper interface {
	StakingTokenSupply(ctx context.Context) (math.Int, error)
	BondedRatio(ctx context.Context) (math.LegacyDec, error)
	ValidatorAddressCodec() address.Codec
	ConsensusAddressCodec() address.Codec
	IterateValidators(ctx context.Context, f func(index int64, validator stakingtypes.ValidatorI) (stop bool)) error
	Validator(ctx context.Context, valAddress sdk.ValAddress) (stakingtypes.ValidatorI, error)
	ValidatorByConsAddr(ctx context.Context, consAddress sdk.ConsAddress) (stakingtypes.ValidatorI, error)
	Delegation(ctx context.Context, address sdk.AccAddress, address2 sdk.ValAddress) (stakingtypes.DelegationI, error)
	IterateDelegations(ctx context.Context, delegator sdk.AccAddress, fn func(index int64, delegation stakingtypes.DelegationI) (stop bool)) error
	GetAllSDKDelegations(ctx context.Context) ([]stakingtypes.Delegation, error)
	GetAllValidators(ctx context.Context) ([]stakingtypes.Validator, error)
	GetAllDelegatorDelegations(ctx context.Context, delegator sdk.AccAddress) ([]stakingtypes.Delegation, error)
	Slash(ctx context.Context, consAddress sdk.ConsAddress, i int64, i2 int64, dec math.LegacyDec) (math.Int, error)
	SlashWithInfractionReason(ctx context.Context, consAddress sdk.ConsAddress, i int64, i2 int64, dec math.LegacyDec, infraction stakingtypes.Infraction) (math.Int, error)
	Jail(ctx context.Context, consAddress sdk.ConsAddress) error
	Unjail(ctx context.Context, consAddress sdk.ConsAddress) error
	MaxValidators(ctx context.Context) (uint32, error)
	IsValidatorJailed(ctx context.Context, addr sdk.ConsAddress) (bool, error)
	ApplyAndReturnValidatorSetUpdates(ctx context.Context) (updates []abci.ValidatorUpdate, err error)
	GetParams(ctx context.Context) (params stakingtypes.Params, err error)
	IterateBondedValidatorsByPower(ctx context.Context, f func(index int64, validator stakingtypes.ValidatorI) (stop bool)) error
	TotalBondedTokens(ctx context.Context) (math.Int, error)
}
