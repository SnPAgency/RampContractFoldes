
<a id="0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp"></a>

# Module `0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31::ramp`



-  [Resource `GlobalStorage`](#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_GlobalStorage)
-  [Struct `AssetAddedEvent`](#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_AssetAddedEvent)
-  [Struct `AssetRemovedEvent`](#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_AssetRemovedEvent)
-  [Struct `ContractStateChangedEvent`](#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_ContractStateChangedEvent)
-  [Struct `OwnerChangedEvent`](#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_OwnerChangedEvent)
-  [Constants](#@Constants_0)
-  [Function `initialize`](#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_initialize)
-  [Function `add_asset`](#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_add_asset)
-  [Function `remove_asset`](#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_remove_asset)
-  [Function `set_contract_state`](#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_set_contract_state)
-  [Function `set_owner`](#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_set_owner)
-  [Function `is_active`](#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_is_active)
-  [Function `get_owner`](#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_get_owner)
-  [Function `is_asset_allowed`](#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_is_asset_allowed)


<pre><code><b>use</b> <a href="">0x1::error</a>;
<b>use</b> <a href="">0x1::event</a>;
<b>use</b> <a href="">0x1::object</a>;
<b>use</b> <a href="">0x1::signer</a>;
<b>use</b> <a href="">0x1::table</a>;
</code></pre>



<a id="0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_GlobalStorage"></a>

## Resource `GlobalStorage`

Global Storage
This resource holds the global state of the RampAptos contract.


<pre><code>#[resource_group_member(#[group = <a href="_ObjectGroup">0x1::object::ObjectGroup</a>])]
<b>struct</b> <a href="ramp.md#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_GlobalStorage">GlobalStorage</a> <b>has</b> key
</code></pre>



<a id="0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_AssetAddedEvent"></a>

## Struct `AssetAddedEvent`

Event emitted when an asset is added


<pre><code>#[<a href="">event</a>]
<b>struct</b> <a href="ramp.md#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_AssetAddedEvent">AssetAddedEvent</a> <b>has</b> drop, store
</code></pre>



<a id="0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_AssetRemovedEvent"></a>

## Struct `AssetRemovedEvent`

Event emitted when an asset is removed


<pre><code>#[<a href="">event</a>]
<b>struct</b> <a href="ramp.md#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_AssetRemovedEvent">AssetRemovedEvent</a> <b>has</b> drop, store
</code></pre>



<a id="0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_ContractStateChangedEvent"></a>

## Struct `ContractStateChangedEvent`

Event emitted when the contract state is changed


<pre><code>#[<a href="">event</a>]
<b>struct</b> <a href="ramp.md#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_ContractStateChangedEvent">ContractStateChangedEvent</a> <b>has</b> drop, store
</code></pre>



<a id="0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_OwnerChangedEvent"></a>

## Struct `OwnerChangedEvent`

Event emitted when the owner is changed


<pre><code>#[<a href="">event</a>]
<b>struct</b> <a href="ramp.md#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_OwnerChangedEvent">OwnerChangedEvent</a> <b>has</b> drop, store
</code></pre>



<a id="@Constants_0"></a>

## Constants


<a id="0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_EASSET_EXISTS"></a>

Error code for the asset already exists in the allowed assets table


<pre><code><b>const</b> <a href="ramp.md#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_EASSET_EXISTS">EASSET_EXISTS</a>: u64 = 3;
</code></pre>



<a id="0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_ENO_ASSET"></a>

Error code for the asset not found in the allowed assets table


<pre><code><b>const</b> <a href="ramp.md#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_ENO_ASSET">ENO_ASSET</a>: u64 = 2;
</code></pre>



<a id="0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_ENO_CONTRACT_STATE"></a>

Errors
These are used to handle errors in the contract.
Error code for the contract state not found


<pre><code><b>const</b> <a href="ramp.md#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_ENO_CONTRACT_STATE">ENO_CONTRACT_STATE</a>: u64 = 0;
</code></pre>



<a id="0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_ENO_OWNER"></a>

Error code for the wrong contract owner as signer


<pre><code><b>const</b> <a href="ramp.md#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_ENO_OWNER">ENO_OWNER</a>: u64 = 1;
</code></pre>



<a id="0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_RAMP_APTOS"></a>

global storage name


<pre><code><b>const</b> <a href="ramp.md#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_RAMP_APTOS">RAMP_APTOS</a>: <a href="">vector</a>&lt;u8&gt; = [82, 65, 77, 80, 95, 65, 80, 84, 79, 83, 95, 71, 76, 79, 66, 65, 76, 95, 83, 84, 79, 82, 65, 71, 69];
</code></pre>



<a id="0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_initialize"></a>

## Function `initialize`



<pre><code><b>public</b> entry <b>fun</b> <a href="ramp.md#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_initialize">initialize</a>(owner: &<a href="">signer</a>, admin: <b>address</b>)
</code></pre>



<a id="0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_add_asset"></a>

## Function `add_asset`



<pre><code><b>public</b> entry <b>fun</b> <a href="ramp.md#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_add_asset">add_asset</a>(owner: &<a href="">signer</a>, asset: <b>address</b>)
</code></pre>



<a id="0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_remove_asset"></a>

## Function `remove_asset`



<pre><code><b>public</b> entry <b>fun</b> <a href="ramp.md#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_remove_asset">remove_asset</a>(owner: &<a href="">signer</a>, asset: <b>address</b>)
</code></pre>



<a id="0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_set_contract_state"></a>

## Function `set_contract_state`



<pre><code><b>public</b> entry <b>fun</b> <a href="ramp.md#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_set_contract_state">set_contract_state</a>(owner: &<a href="">signer</a>, state: bool)
</code></pre>



<a id="0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_set_owner"></a>

## Function `set_owner`



<pre><code><b>public</b> entry <b>fun</b> <a href="ramp.md#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_set_owner">set_owner</a>(owner: &<a href="">signer</a>, new_owner: <b>address</b>)
</code></pre>



<a id="0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_is_active"></a>

## Function `is_active`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="ramp.md#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_is_active">is_active</a>(): bool
</code></pre>



<a id="0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_get_owner"></a>

## Function `get_owner`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="ramp.md#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_get_owner">get_owner</a>(): <b>address</b>
</code></pre>



<a id="0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_is_asset_allowed"></a>

## Function `is_asset_allowed`



<pre><code>#[view]
<b>public</b> <b>fun</b> <a href="ramp.md#0x70c1e9dc6500d8cf161a4716302924dde0d198478a65adc8cff27805b1107a31_ramp_is_asset_allowed">is_asset_allowed</a>(asset: <b>address</b>): bool
</code></pre>
