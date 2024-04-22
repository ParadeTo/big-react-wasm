/**
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 *
 * @emails react-core
 */

'use strict';

let React;
let ReactDOM;
let ReactTestUtils;

describe('ReactElement', () => {
    let ComponentFC;
    let originalSymbol;

    beforeEach(() => {
        jest.resetModules();

        // Delete the native Symbol if we have one to ensure we test the
        // unpolyfilled environment.
        originalSymbol = global.Symbol;
        global.Symbol = undefined;

        React = require('../../dist/react');
        ReactDOM = require('../../dist/react-dom');
        // ReactTestUtils = require('react-dom/test-utils');

        // NOTE: We're explicitly not using JSX here. This is intended to test
        // classic JS without JSX.
        ComponentFC = () => {
            return React.createElement('div');
        };
    });

    afterEach(() => {
        global.Symbol = originalSymbol;
    });

    it('uses the fallback value when in an environment without Symbol', () => {
        expect((<div/>).$$typeof).toBe('react.element');
    });
})