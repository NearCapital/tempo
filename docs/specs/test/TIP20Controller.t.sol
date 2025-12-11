// SPDX-License-Identifier: MIT
pragma solidity ^0.8.13;

import { TIP20 } from "../src/TIP20.sol";
import { TIP20Controller } from "../src/TIP20Controller.sol";
import { ITIP20 } from "../src/interfaces/ITIP20.sol";
import { ITIP20RolesAuth } from "../src/interfaces/ITIP20RolesAuth.sol";
import { BaseTest } from "./BaseTest.t.sol";

contract TIP20ControllerTest is BaseTest {

    TIP20Controller controller;
    TIP20 ledgerToken;
    TIP20 wrappedToken;

    bytes32 constant MINTER_ROLE = keccak256("MINTER_ROLE");

    function setUp() public override {
        super.setUp();

        // Create ledger token (backing token) and wrapped token (pathUSD-like)
        ledgerToken =
            TIP20(factory.createToken("Ledger USD", "LUSD", "USD", TIP20(_PATH_USD), admin));
        wrappedToken =
            TIP20(factory.createToken("Wrapped USD", "WUSD", "USD", TIP20(_PATH_USD), admin));

        // Deploy controller with admin
        controller =
            new TIP20Controller(ITIP20(address(ledgerToken)), ITIP20(address(wrappedToken)), admin);

        // Grant ISSUER_ROLE on wrappedToken to the controller so it can mint/burn
        vm.prank(admin);
        wrappedToken.grantRole(_ISSUER_ROLE, address(controller));

        // Grant MINTER_ROLE on controller to alice (she will be our minter)
        vm.prank(admin);
        controller.grantRole(MINTER_ROLE, alice);

        // Mint some ledger tokens to alice for testing
        vm.prank(admin);
        ledgerToken.grantRole(_ISSUER_ROLE, admin);
        vm.prank(admin);
        ledgerToken.mint(alice, 1000e6);
    }

    // ========== MINT FLOW TESTS ==========

    function test_mint_success() public {
        uint256 amount = 100e6;

        // Alice approves controller to spend her ledger tokens
        vm.prank(alice);
        ledgerToken.approve(address(controller), amount);

        // Check balances before
        uint256 aliceLedgerBefore = ledgerToken.balanceOf(alice);
        uint256 aliceWrappedBefore = wrappedToken.balanceOf(alice);
        uint256 controllerLedgerBefore = ledgerToken.balanceOf(address(controller));

        // Alice mints wrapped tokens
        vm.prank(alice);
        controller.mint(amount);

        // Check balances after
        assertEq(
            ledgerToken.balanceOf(alice),
            aliceLedgerBefore - amount,
            "Alice ledger balance should decrease"
        );
        assertEq(
            wrappedToken.balanceOf(alice),
            aliceWrappedBefore + amount,
            "Alice wrapped balance should increase"
        );
        assertEq(
            ledgerToken.balanceOf(address(controller)),
            controllerLedgerBefore + amount,
            "Controller should hold ledger tokens"
        );
    }

    function test_mint_revertsWithoutMinterRole() public {
        uint256 amount = 100e6;

        // Bob doesn't have MINTER_ROLE
        vm.prank(bob);
        ledgerToken.approve(address(controller), amount);

        vm.prank(bob);
        try controller.mint(amount) {
            revert CallShouldHaveReverted();
        } catch (bytes memory err) {
            assertEq(err, abi.encodeWithSelector(ITIP20RolesAuth.Unauthorized.selector));
        }
    }

    function test_mint_revertsWithoutApproval() public {
        uint256 amount = 100e6;

        // Alice has MINTER_ROLE but hasn't approved
        vm.prank(alice);
        try controller.mint(amount) {
            revert CallShouldHaveReverted();
        } catch (bytes memory err) {
            assertEq(err, abi.encodeWithSelector(ITIP20.InsufficientAllowance.selector));
        }
    }

    // ========== BURN FLOW TESTS ==========

    function test_burn_success() public {
        uint256 mintAmount = 100e6;
        uint256 burnAmount = 50e6;

        // First mint some wrapped tokens
        vm.prank(alice);
        ledgerToken.approve(address(controller), mintAmount);
        vm.prank(alice);
        controller.mint(mintAmount);

        // Alice approves controller to spend her wrapped tokens
        vm.prank(alice);
        wrappedToken.approve(address(controller), burnAmount);

        // Check balances before burn
        uint256 aliceLedgerBefore = ledgerToken.balanceOf(alice);
        uint256 aliceWrappedBefore = wrappedToken.balanceOf(alice);
        uint256 controllerLedgerBefore = ledgerToken.balanceOf(address(controller));

        // Alice burns wrapped tokens
        vm.prank(alice);
        controller.burn(burnAmount);

        // Check balances after
        assertEq(
            ledgerToken.balanceOf(alice),
            aliceLedgerBefore + burnAmount,
            "Alice ledger balance should increase"
        );
        assertEq(
            wrappedToken.balanceOf(alice),
            aliceWrappedBefore - burnAmount,
            "Alice wrapped balance should decrease"
        );
        assertEq(
            ledgerToken.balanceOf(address(controller)),
            controllerLedgerBefore - burnAmount,
            "Controller ledger balance should decrease"
        );
    }

    function test_burn_revertsWithoutMinterRole() public {
        uint256 amount = 100e6;

        // First mint some wrapped tokens to bob (via admin granting role temporarily)
        vm.prank(admin);
        controller.grantRole(MINTER_ROLE, bob);

        vm.prank(admin);
        ledgerToken.mint(bob, amount);

        vm.prank(bob);
        ledgerToken.approve(address(controller), amount);
        vm.prank(bob);
        controller.mint(amount);

        // Revoke bob's MINTER_ROLE
        vm.prank(admin);
        controller.revokeRole(MINTER_ROLE, bob);

        // Bob tries to burn without MINTER_ROLE
        vm.prank(bob);
        wrappedToken.approve(address(controller), amount);

        vm.prank(bob);
        try controller.burn(amount) {
            revert CallShouldHaveReverted();
        } catch (bytes memory err) {
            assertEq(err, abi.encodeWithSelector(ITIP20RolesAuth.Unauthorized.selector));
        }
    }

    function test_burn_revertsWithoutApproval() public {
        uint256 amount = 100e6;

        // First mint some wrapped tokens
        vm.prank(alice);
        ledgerToken.approve(address(controller), amount);
        vm.prank(alice);
        controller.mint(amount);

        // Alice tries to burn without approving wrapped tokens
        vm.prank(alice);
        try controller.burn(amount) {
            revert CallShouldHaveReverted();
        } catch (bytes memory err) {
            assertEq(err, abi.encodeWithSelector(ITIP20.InsufficientAllowance.selector));
        }
    }

    // ========== E2E FLOW TESTS ==========

    function test_e2e_mintAndBurnFullCycle() public {
        uint256 amount = 500e6;

        // Initial state
        assertEq(ledgerToken.balanceOf(alice), 1000e6, "Alice starts with 1000 ledger tokens");
        assertEq(wrappedToken.balanceOf(alice), 0, "Alice starts with 0 wrapped tokens");
        assertEq(
            ledgerToken.balanceOf(address(controller)), 0, "Controller starts with 0 ledger tokens"
        );

        // Step 1: Alice mints wrapped tokens
        vm.prank(alice);
        ledgerToken.approve(address(controller), amount);
        vm.prank(alice);
        controller.mint(amount);

        assertEq(ledgerToken.balanceOf(alice), 500e6, "Alice has 500 ledger tokens after mint");
        assertEq(wrappedToken.balanceOf(alice), 500e6, "Alice has 500 wrapped tokens after mint");
        assertEq(
            ledgerToken.balanceOf(address(controller)), 500e6, "Controller holds 500 ledger tokens"
        );

        // Step 2: Alice burns all wrapped tokens
        vm.prank(alice);
        wrappedToken.approve(address(controller), amount);
        vm.prank(alice);
        controller.burn(amount);

        assertEq(ledgerToken.balanceOf(alice), 1000e6, "Alice has 1000 ledger tokens after burn");
        assertEq(wrappedToken.balanceOf(alice), 0, "Alice has 0 wrapped tokens after burn");
        assertEq(ledgerToken.balanceOf(address(controller)), 0, "Controller holds 0 ledger tokens");
    }

    function test_e2e_multipleMintAndBurn() public {
        // Mint in batches
        vm.prank(alice);
        ledgerToken.approve(address(controller), 300e6);

        vm.prank(alice);
        controller.mint(100e6);
        assertEq(wrappedToken.balanceOf(alice), 100e6);

        vm.prank(alice);
        controller.mint(200e6);
        assertEq(wrappedToken.balanceOf(alice), 300e6);

        // Burn in batches
        vm.prank(alice);
        wrappedToken.approve(address(controller), 300e6);

        vm.prank(alice);
        controller.burn(150e6);
        assertEq(wrappedToken.balanceOf(alice), 150e6);
        assertEq(ledgerToken.balanceOf(alice), 850e6);

        vm.prank(alice);
        controller.burn(150e6);
        assertEq(wrappedToken.balanceOf(alice), 0);
        assertEq(ledgerToken.balanceOf(alice), 1000e6);
    }

    function test_e2e_multipleMintersCanOperate() public {
        // Grant MINTER_ROLE to bob
        vm.prank(admin);
        controller.grantRole(MINTER_ROLE, bob);

        // Give bob some ledger tokens
        vm.prank(admin);
        ledgerToken.mint(bob, 500e6);

        // Alice mints
        vm.prank(alice);
        ledgerToken.approve(address(controller), 200e6);
        vm.prank(alice);
        controller.mint(200e6);

        // Bob mints
        vm.prank(bob);
        ledgerToken.approve(address(controller), 300e6);
        vm.prank(bob);
        controller.mint(300e6);

        // Verify independent balances
        assertEq(wrappedToken.balanceOf(alice), 200e6);
        assertEq(wrappedToken.balanceOf(bob), 300e6);
        assertEq(ledgerToken.balanceOf(address(controller)), 500e6);

        // Both can burn independently
        vm.prank(alice);
        wrappedToken.approve(address(controller), 200e6);
        vm.prank(alice);
        controller.burn(200e6);

        vm.prank(bob);
        wrappedToken.approve(address(controller), 300e6);
        vm.prank(bob);
        controller.burn(300e6);

        assertEq(wrappedToken.balanceOf(alice), 0);
        assertEq(wrappedToken.balanceOf(bob), 0);
        assertEq(ledgerToken.balanceOf(address(controller)), 0);
    }

    // ========== ROLE MANAGEMENT TESTS ==========

    function test_adminCanGrantAndRevokeMinterRole() public {
        assertFalse(controller.hasRole(charlie, MINTER_ROLE));

        // Grant role
        vm.prank(admin);
        controller.grantRole(MINTER_ROLE, charlie);
        assertTrue(controller.hasRole(charlie, MINTER_ROLE));

        // Revoke role
        vm.prank(admin);
        controller.revokeRole(MINTER_ROLE, charlie);
        assertFalse(controller.hasRole(charlie, MINTER_ROLE));
    }

    function test_nonAdminCannotGrantMinterRole() public {
        vm.prank(alice);
        try controller.grantRole(MINTER_ROLE, bob) {
            revert CallShouldHaveReverted();
        } catch (bytes memory err) {
            assertEq(err, abi.encodeWithSelector(ITIP20RolesAuth.Unauthorized.selector));
        }
    }

}
