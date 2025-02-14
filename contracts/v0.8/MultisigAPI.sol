/*******************************************************************************
 *   (c) 2022 Zondax AG
 *
 *  Licensed under the Apache License, Version 2.0 (the "License");
 *  you may not use this file except in compliance with the License.
 *  You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS,
 *  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 ********************************************************************************/
//
// DRAFT!! THIS CODE HAS NOT BEEN AUDITED - USE ONLY FOR PROTOTYPING
//
pragma solidity >=0.4.25 <=0.8.15;

import "./types/MultisigTypes.sol";
import "./cbor/MultisigCbor.sol";
import "./types/CommonTypes.sol";
import "./utils/Misc.sol";
import "./utils/Actor.sol";

/// @title This contract is a proxy to the Multisig actor. Calling one of its methods will result in a cross-actor call being performed.
/// @author Zondax AG
contract MultisigAPI {
    using BytesCBOR for bytes;
    using ProposeCBOR for MultisigTypes.ProposeParams;
    using ProposeCBOR for MultisigTypes.ProposeReturn;
    using TxnIDCBOR for MultisigTypes.TxnIDParams;
    using ApproveCBOR for MultisigTypes.ApproveReturn;
    using AddSignerCBOR for MultisigTypes.AddSignerParams;
    using RemoveSignerCBOR for MultisigTypes.RemoveSignerParams;
    using SwapSignerCBOR for MultisigTypes.SwapSignerParams;
    using ChangeNumApprovalsThresholdCBOR for MultisigTypes.ChangeNumApprovalsThresholdParams;
    using LockBalanceCBOR for MultisigTypes.LockBalanceParams;

    /// @notice FIXME
    function propose(bytes memory target, MultisigTypes.ProposeParams memory params) public returns (MultisigTypes.ProposeReturn memory) {
        bytes memory raw_request = params.serialize();

        bytes memory raw_response = Actor.call(MultisigTypes.ProposeMethodNum, target, raw_request);

        bytes memory result = Actor.readRespData(raw_response);

        MultisigTypes.ProposeReturn memory response;
        response.deserialize(result);

        return response;
    }

    /// @notice FIXME
    function approve(bytes memory target, MultisigTypes.TxnIDParams memory params) public returns (MultisigTypes.ApproveReturn memory) {
        bytes memory raw_request = params.serialize();

        bytes memory raw_response = Actor.call(MultisigTypes.ApproveMethodNum, target, raw_request);

        bytes memory result = Actor.readRespData(raw_response);

        MultisigTypes.ApproveReturn memory response;
        response.deserialize(result);

        return response;
    }

    /// @notice FIXME
    function cancel(bytes memory target, MultisigTypes.TxnIDParams memory params) public {
        bytes memory raw_request = params.serialize();

        bytes memory raw_response = Actor.call(MultisigTypes.CancelMethodNum, target, raw_request);

        bytes memory result = Actor.readRespData(raw_response);
    }

    /// @notice FIXME
    function add_signer(bytes memory target, MultisigTypes.AddSignerParams memory params) public {
        bytes memory raw_request = params.serialize();

        bytes memory raw_response = Actor.call(MultisigTypes.AddSignerMethodNum, target, raw_request);

        bytes memory result = Actor.readRespData(raw_response);
    }

    /// @notice FIXME
    function remove_signer(bytes memory target, MultisigTypes.RemoveSignerParams memory params) public {
        bytes memory raw_request = params.serialize();

        bytes memory raw_response = Actor.call(MultisigTypes.RemoveSignerMethodNum, target, raw_request);

        bytes memory result = Actor.readRespData(raw_response);
    }

    /// @notice FIXME
    function swap_signer(bytes memory target, MultisigTypes.SwapSignerParams memory params) public {
        bytes memory raw_request = params.serialize();

        bytes memory raw_response = Actor.call(MultisigTypes.SwapSignerMethodNum, target, raw_request);

        bytes memory result = Actor.readRespData(raw_response);
    }

    /// @notice FIXME
    function swap_signer(bytes memory target, MultisigTypes.ChangeNumApprovalsThresholdParams memory params) public {
        bytes memory raw_request = params.serialize();

        bytes memory raw_response = Actor.call(MultisigTypes.ChangeNumApprovalsThresholdMethodNum, target, raw_request);

        bytes memory result = Actor.readRespData(raw_response);
    }

    /// @notice FIXME
    function lock_balance(bytes memory target, MultisigTypes.LockBalanceParams memory params) public {
        bytes memory raw_request = params.serialize();

        bytes memory raw_response = Actor.call(MultisigTypes.LockBalanceMethodNum, target, raw_request);

        bytes memory result = Actor.readRespData(raw_response);
    }

    /// @notice FIXME
    function universal_receiver_hook(bytes memory target, bytes memory params) public {
        bytes memory raw_request = params.serializeBytes();

        bytes memory raw_response = Actor.call(MultisigTypes.UniversalReceiverHookMethodNum, target, raw_request);

        bytes memory result = Actor.readRespData(raw_response);
    }
}
