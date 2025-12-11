// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import { TIP20RolesAuth } from "./abstracts/TIP20RolesAuth.sol";
import { ITIP20 } from "./interfaces/ITIP20.sol";

/// @title TIP20 Wrapping Controller
/// @notice A controller contract that wraps a ledger TIP20 token into a wrapped TIP20 token (e.g., pathUSD) with 1:1 ratio.
/// @dev The controller must have ISSUER_ROLE on the wrapped token to mint and burn.
contract TIP20Controller is TIP20RolesAuth {

    error TransferFailed();

    /// @notice Role required to mint and burn via this controller
    bytes32 public constant MINTER_ROLE = keccak256("MINTER_ROLE");

    /// @notice The backing ledger token (TIP20)
    ITIP20 public immutable LEDGER_TOKEN;

    /// @notice The wrapped token (e.g., pathUSD)
    ITIP20 public immutable WRAPPED_TOKEN;

    /// @notice Emitted when wrapped tokens are minted
    event Mint(address indexed minter, uint256 amount);

    /// @notice Emitted when wrapped tokens are burned
    event Burn(address indexed burner, uint256 amount);

    /// @param ledgerToken The backing ledger token address
    /// @param wrappedToken The wrapped token address (e.g., pathUSD)
    /// @param admin The initial admin who can grant MINTER_ROLE
    constructor(ITIP20 ledgerToken, ITIP20 wrappedToken, address admin) {
        LEDGER_TOKEN = ledgerToken;
        WRAPPED_TOKEN = wrappedToken;
        hasRole[admin][DEFAULT_ADMIN_ROLE] = true;
    }

    /// @notice Mints wrapped tokens by depositing ledger tokens
    /// @dev Caller must have approved this contract to spend their ledger tokens
    /// @param amount The amount of tokens to mint (1:1 ratio with ledger tokens)
    function mint(uint256 amount) external onlyRole(MINTER_ROLE) {
        // Pull ledger tokens from minter to this contract
        if (!LEDGER_TOKEN.transferFrom(msg.sender, address(this), amount)) revert TransferFailed();

        // Mint wrapped tokens to minter
        WRAPPED_TOKEN.mint(msg.sender, amount);

        emit Mint(msg.sender, amount);
    }

    /// @notice Burns wrapped tokens and returns ledger tokens
    /// @dev Caller must have approved this contract to spend their wrapped tokens
    /// @param amount The amount of wrapped tokens to burn (1:1 ratio with ledger tokens returned)
    function burn(uint256 amount) external onlyRole(MINTER_ROLE) {
        // Pull wrapped tokens from burner to this contract
        if (!WRAPPED_TOKEN.transferFrom(msg.sender, address(this), amount)) {
            revert TransferFailed();
        }

        // Burn the wrapped tokens from this contract's balance
        WRAPPED_TOKEN.burn(amount);

        // Transfer ledger tokens back to burner
        if (!LEDGER_TOKEN.transfer(msg.sender, amount)) revert TransferFailed();

        emit Burn(msg.sender, amount);
    }

}
