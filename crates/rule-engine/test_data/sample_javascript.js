// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

// Sample JavaScript code for benchmarking
function testFunction() {
    console.log("Hello World");
    console.log('test string');
    console.log(`template ${variable}`);
}

class TestClass {
    constructor() {
        this.value = 42;
    }

    method() {
        console.log(this.value);
    }
}

let variable = "test";
const constant = 123;
var oldVar = true;

import { Component } from 'react';
import * as React from 'react';
import defaultExport from './module';

async function asyncFunction() {
    const result = await fetch('/api/data');
    return result.json();
}

function recursion() {
    recursion();
}

let recursion2 = () => {
    recursion2();
};

export default TestClass;
