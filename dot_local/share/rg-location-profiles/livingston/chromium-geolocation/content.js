(function () {
  "use strict";

    var profile = {
      latitude: 30.7110,
      longitude: -94.9327,
      accuracy: 35,
      timezone: "America/Chicago",
      locale: "en-US",
      languages: ["en-US", "en"]
    };

    var NativeDateTimeFormat = Intl.DateTimeFormat;
    var nativeResolvedOptions = Intl.DateTimeFormat.prototype.resolvedOptions;
    var nativeGetTimezoneOffset = Date.prototype.getTimezoneOffset;

    function defineGetter(target, key, getter) {
      try {
        Object.defineProperty(target, key, {
          get: getter,
          configurable: true
        });
      } catch (error) {
        // Some pages harden built-ins. The process TZ still carries the
        // profile for browser APIs that read the OS environment.
      }
    }

    function timezoneOffsetMinutes(date) {
      var parts = new NativeDateTimeFormat("en-US", {
        timeZone: profile.timezone,
        year: "numeric",
        month: "2-digit",
        day: "2-digit",
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
        hourCycle: "h23"
      }).formatToParts(date).reduce(function (acc, part) {
        if (part.type !== "literal") {
          acc[part.type] = part.value;
        }
        return acc;
      }, {});

      var localAsUtc = Date.UTC(
        Number(parts.year),
        Number(parts.month) - 1,
        Number(parts.day),
        Number(parts.hour),
        Number(parts.minute),
        Number(parts.second)
      );
      return Math.round((date.getTime() - localAsUtc) / 60000);
    }

    function position() {
      return {
        coords: {
          latitude: profile.latitude,
          longitude: profile.longitude,
          accuracy: profile.accuracy,
          altitude: null,
          altitudeAccuracy: null,
          heading: null,
          speed: null
        },
        timestamp: Date.now()
      };
    }

    var watches = new Map();
    var nextWatchId = 1;
    var geolocation = {
      getCurrentPosition: function (success, error, options) {
        void error;
        void options;
        if (typeof success === "function") {
          setTimeout(function () {
            success(position());
          }, 0);
        }
      },
      watchPosition: function (success, error, options) {
        void error;
        void options;
        var id = nextWatchId++;
        if (typeof success === "function") {
          watches.set(id, setInterval(function () {
            success(position());
          }, 1000));
          setTimeout(function () {
            success(position());
          }, 0);
        }
        return id;
      },
      clearWatch: function (id) {
        if (watches.has(id)) {
          clearInterval(watches.get(id));
          watches.delete(id);
        }
      }
    };

    defineGetter(Navigator.prototype, "language", function () {
      return profile.locale;
    });
    defineGetter(Navigator.prototype, "languages", function () {
      return profile.languages.slice();
    });
    defineGetter(Navigator.prototype, "geolocation", function () {
      return geolocation;
    });

    try {
      Intl.DateTimeFormat.prototype.resolvedOptions = function () {
        var options = nativeResolvedOptions.call(this);
        options.locale = profile.locale;
        options.timeZone = profile.timezone;
        return options;
      };
    } catch (error) {
      // Ignore hardened Intl objects.
    }

    try {
      Date.prototype.getTimezoneOffset = function () {
        try {
          return timezoneOffsetMinutes(this);
        } catch (error) {
          return nativeGetTimezoneOffset.call(this);
        }
      };
    } catch (error) {
      // Ignore hardened Date objects.
    }

    if (navigator.permissions && navigator.permissions.query) {
      try {
        var nativeQuery = navigator.permissions.query.bind(navigator.permissions);
        navigator.permissions.query = function (descriptor) {
          if (descriptor && descriptor.name === "geolocation") {
            return Promise.resolve({
              state: "granted",
              onchange: null,
              addEventListener: function () {},
              removeEventListener: function () {},
              dispatchEvent: function () {
                return true;
              }
            });
          }
          return nativeQuery(descriptor);
        };
      } catch (error) {
        // Permission state is advisory; geolocation is overridden above.
      }
    }
})();
