// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

library TIP20UserStorage {

    /// @notice State variables associated with each user address.
    struct UserInfo {
        uint256 balance;
        address rewardRecipient;
        uint256 rewardPerToken;
        uint256 rewardBalance;
    }

    /// @notice Gets the TIP20 token balance for a user
    function getBalance(address user) internal view returns (uint256 val) {
        assembly ("memory-safe") {
            let slot := shl(96, user)
            val := sload(slot)
        }
    }

    /// @notice Sets the TIP20 token balance for a user
    function setBalance(address user, uint256 val) internal {
        assembly ("memory-safe") {
            let slot := shl(96, user)
            sstore(slot, val)
        }
    }

    /// @notice Gets the reward recipient for a user
    function getRewardRecipient(address user) internal view returns (address recipient) {
        assembly ("memory-safe") {
            let slot := add(shl(96, user), 32)
            recipient := sload(slot)
        }
    }

    /// @notice Sets the reward recipient for a user
    /// @dev Assumes that the dirty upper bits for `recipient` have been cleaned
    function setRewardRecipient(address user, address recipient) internal {
        assembly ("memory-safe") {
            let slot := add(shl(96, user), 32)
            sstore(slot, recipient)
        }
    }

    /// @notice Gets the reward per token for a user
    function getRewardPerToken(address user) internal view returns (uint256 val) {
        assembly ("memory-safe") {
            let slot := add(shl(96, user), 64)
            val := sload(slot)
        }
    }

    /// @notice Sets the reward per token for a user
    function setRewardPerToken(address user, uint256 val) internal {
        assembly ("memory-safe") {
            let slot := add(shl(96, user), 64)
            sstore(slot, val)
        }
    }

    /// @notice Gets the reward balance for a user
    function getRewardBalance(address user) internal view returns (uint256 val) {
        assembly ("memory-safe") {
            let slot := add(shl(96, user), 96)
            val := sload(slot)
        }
    }

    /// @notice Sets the reward balance for a user
    function setRewardBalance(address user, uint256 val) internal {
        assembly ("memory-safe") {
            let slot := add(shl(96, user), 96)
            sstore(slot, val)
        }
    }

}
