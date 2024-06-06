Object.setPrototypeOf(window, Window.prototype)

expect.extend({
    ...require('./reactTestMatchers')
});
