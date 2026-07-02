import os
import sys
import argparse

from xrpl.account import get_balance
from xrpl.clients import JsonRpcClient
from xrpl.wallet import Wallet, generate_faucet_wallet

DEFAULT_COUNT = 3
TESTNET_URL = "https://s.altnet.rippletest.net:51234/"
MIN_BALANCE_XRP = 100


def has_sufficient_balance(client, address, min_balance_xrp):
    try:
        return get_balance(address, client) > min_balance_xrp * 1_000_000
    except Exception:
        return False


def wallet_from_env(seed_name):
    seed = os.getenv(seed_name)
    if not seed:
        print(f"Error: {seed_name} not found in environment.")
        sys.exit(1)
    return Wallet.from_seed(seed)


def fund_account(client, seed_name, min_balance_xrp):
    wallet = wallet_from_env(seed_name)

    if has_sufficient_balance(client, wallet.classic_address, min_balance_xrp):
        print(f"Skipping {seed_name} ({wallet.classic_address}): balance sufficient")
        return

    try:
        generate_faucet_wallet(client, wallet)
        print(f"Funded {seed_name} ({wallet.classic_address})")
    except Exception as e:
        print(f"Failed to fund {seed_name} ({wallet.classic_address}): {e}")
        sys.exit(1)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Fund XRPL test accounts via faucet.")
    parser.add_argument("--count", type=int, default=DEFAULT_COUNT, help="Fund TEST_SEED_1 through TEST_SEED_<count> (default: %(default)s)")
    parser.add_argument("--url", default=TESTNET_URL, help="Testnet JSON-RPC URL (default: %(default)s)")
    parser.add_argument("--min-balance", type=int, default=MIN_BALANCE_XRP, help="Minimum balance in XRP before skipping (default: %(default)s)")
    args = parser.parse_args()

    client = JsonRpcClient(args.url)
    for i in range(1, args.count + 1):
        fund_account(client, f"TEST_SEED_{i}", args.min_balance)
