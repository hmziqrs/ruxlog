/**
 * Firebase Analytics initialization and helper functions
 *
 * This module provides a bridge between Rust/WASM and Firebase JS SDK.
 * Loaded via Dioxus.toml configuration.
 */

let firebaseApp = null;
let analytics = null;

/**
 * Initialize Firebase with the provided configuration
 * @param {string} apiKey
 * @param {string} authDomain
 * @param {string} projectId
 * @param {string} storageBucket
 * @param {string} messagingSenderId
 * @param {string} appId
 * @param {string} measurementId
 * @returns {boolean} true if initialization was successful
 */
export function initFirebase(
    apiKey,
    authDomain,
    projectId,
    storageBucket,
    messagingSenderId,
    appId,
    measurementId
) {
    try {
        // Check if Firebase is already initialized
        if (firebaseApp) {
            console.log('Firebase already initialized');
            return true;
        }

        // Validate that we have the necessary values
        if (!apiKey || !projectId || !appId) {
            console.error('Missing required Firebase configuration');
            return false;
        }

        // Check if Firebase SDK is loaded
        if (typeof firebase === 'undefined') {
            console.error('Firebase SDK not loaded. Make sure firebase scripts are included in index.html');
            return false;
        }

        const firebaseConfig = {
            apiKey: apiKey,
            authDomain: authDomain,
            projectId: projectId,
            storageBucket: storageBucket,
            messagingSenderId: messagingSenderId,
            appId: appId,
            measurementId: measurementId
        };

        // Initialize Firebase
        firebaseApp = firebase.initializeApp(firebaseConfig);

        // Initialize Analytics
        analytics = firebase.analytics();

        console.log('Firebase Analytics initialized successfully');
        return true;
    } catch (error) {
        console.error('Error initializing Firebase:', error);
        return false;
    }
}

/**
 * Check if Firebase Analytics is available
 * @returns {boolean}
 */
export function isAnalyticsAvailable() {
    return analytics !== null;
}

/**
 * Log a custom event with optional parameters
 * @param {string} eventName
 * @param {Object} params
 */
export function logAnalyticsEvent(eventName, params) {
    if (!analytics) {
        console.warn('Analytics not initialized');
        return;
    }

    try {
        analytics.logEvent(eventName, params);
    } catch (error) {
        console.error('Error logging event:', error);
    }
}

/**
 * Log a page view event
 * @param {string} pagePath
 * @param {string} pageTitle
 */
export function logPageView(pagePath, pageTitle) {
    if (!analytics) {
        console.warn('Analytics not initialized');
        return;
    }

    try {
        analytics.logEvent('page_view', {
            page_path: pagePath,
            page_title: pageTitle
        });
    } catch (error) {
        console.error('Error logging page view:', error);
    }
}

/**
 * Set a user property
 * @param {string} name
 * @param {string} value
 */
export function setUserProperty(name, value) {
    if (!analytics) {
        console.warn('Analytics not initialized');
        return;
    }

    try {
        analytics.setUserProperties({
            [name]: value
        });
    } catch (error) {
        console.error('Error setting user property:', error);
    }
}

/**
 * Set the current screen/page name
 * @param {string} screenName
 */
export function setCurrentScreen(screenName) {
    if (!analytics) {
        console.warn('Analytics not initialized');
        return;
    }

    try {
        analytics.setCurrentScreen(screenName);
    } catch (error) {
        console.error('Error setting current screen:', error);
    }
}
