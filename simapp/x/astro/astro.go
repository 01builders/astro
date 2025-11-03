package astro

import (
	"context"

	abci "github.com/cometbft/cometbft/abci/types"
)

type Keeper interface {
	ApplyAndReturnValidatorSetUpdates(ctx context.Context) (updates []abci.ValidatorUpdate, err error)
}
