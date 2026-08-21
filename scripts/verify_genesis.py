import json
import sys
from decimal import Decimal, getcontext

getcontext().prec = 60

class bcolors:
    OKGREEN = '\033[92m'
    WARNING = '\033[93m'
    FAIL = '\033[91m'
    ENDC = '\033[0m'

def print_pass(msg):
    print(f"{bcolors.OKGREEN}[PASS]{bcolors.ENDC} {msg}")

def print_fail(msg):
    print(f"{bcolors.FAIL}[FAIL]{bcolors.ENDC} {msg}")

def print_warn(msg):
    print(f"{bcolors.WARNING}[WARN]{bcolors.ENDC} {msg}")

def main():
    try:
        with open('d:/sprax-chain/deploy/genesis/genesis.json', 'r') as f:
            genesis = json.load(f)
    except FileNotFoundError:
        print_fail("genesis.json not found")
        sys.exit(1)

    errors = 0

    # Verify chainId
    if genesis.get('chainId') == 'sprax-mainnet-1':
        print_pass("Chain ID is sprax-mainnet-1")
    else:
        print_fail(f"Invalid chainId: {genesis.get('chainId')}")
        errors += 1

    # Verify totalSupplyAtto
    total_supply_atto_expected = Decimal("1000000000000000000000000000")
    if Decimal(genesis.get('totalSupplyAtto')) == total_supply_atto_expected:
        print_pass("Total supply Atto is 10^27")
    else:
        print_fail("Total supply Atto mismatch")
        errors += 1

    # Verify allocations
    allocations = genesis.get('allocationSchedule', {})
    sum_atto = Decimal("0")
    sum_pct = Decimal("0")

    for name, alloc in allocations.items():
        amount_sprx = Decimal(alloc['amountSprx'])
        amount_atto = Decimal(alloc['amountAtto'])
        pct = Decimal(alloc['pct'])

        if amount_sprx * Decimal("1000000000000000000") == amount_atto:
            print_pass(f"Allocation {name}: SPRX to Atto conversion correct")
        else:
            print_fail(f"Allocation {name}: SPRX to Atto conversion failed")
            errors += 1
            
        sum_atto += amount_atto
        sum_pct += pct

    if sum_atto == total_supply_atto_expected:
        print_pass("Allocations sum to total supply Atto")
    else:
        print_fail(f"Allocations sum mismatch. Got {sum_atto}")
        errors += 1

    if sum_pct == Decimal("100.0"):
        print_pass("Allocations pct sum to 100.0")
    else:
        print_fail(f"Allocations pct sum mismatch. Got {sum_pct}")
        errors += 1

    # Verify consensus params
    consensus = genesis.get('consensusParams', {})
    if consensus.get('blockTimeTargetMs') == 1500:
        print_pass("blockTimeTargetMs is 1500")
    else:
        print_fail("blockTimeTargetMs mismatch")
        errors += 1

    if consensus.get('maxBlockGas') == 20000000:
        print_pass("maxBlockGas is 20000000")
    else:
        print_fail("maxBlockGas mismatch")
        errors += 1

    # Verify staking params
    staking = genesis.get('stakingParams', {})
    if staking.get('maxValidators') == 100:
        print_pass("maxValidators is 100")
    else:
        print_fail("maxValidators mismatch")
        errors += 1

    if staking.get('minSelfDelegationAtto') == "1000000000000000000000":
        print_pass("minSelfDelegation is 1000 SPRX")
    else:
        print_fail("minSelfDelegation mismatch")
        errors += 1

    # Check placeholders
    for val in genesis.get('genesisValidators', []):
        if "PLACEHOLDER" in val.get('consensusPubkey', ''):
            print_warn(f"Validator {val.get('moniker')} has placeholder pubkey")

    if errors > 0:
        print(f"\n{bcolors.FAIL}Genesis verification failed with {errors} errors.{bcolors.ENDC}")
        sys.exit(1)
    else:
        print(f"\n{bcolors.OKGREEN}Genesis verification passed successfully.{bcolors.ENDC}")
        sys.exit(0)

if __name__ == '__main__':
    main()
