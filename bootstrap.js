/******/ (function(modules) { // webpackBootstrap
/******/ 	// install a JSONP callback for chunk loading
/******/ 	function webpackJsonpCallback(data) {
/******/ 		var chunkIds = data[0];
/******/ 		var moreModules = data[1];
/******/
/******/
/******/ 		// add "moreModules" to the modules object,
/******/ 		// then flag all "chunkIds" as loaded and fire callback
/******/ 		var moduleId, chunkId, i = 0, resolves = [];
/******/ 		for(;i < chunkIds.length; i++) {
/******/ 			chunkId = chunkIds[i];
/******/ 			if(Object.prototype.hasOwnProperty.call(installedChunks, chunkId) && installedChunks[chunkId]) {
/******/ 				resolves.push(installedChunks[chunkId][0]);
/******/ 			}
/******/ 			installedChunks[chunkId] = 0;
/******/ 		}
/******/ 		for(moduleId in moreModules) {
/******/ 			if(Object.prototype.hasOwnProperty.call(moreModules, moduleId)) {
/******/ 				modules[moduleId] = moreModules[moduleId];
/******/ 			}
/******/ 		}
/******/ 		if(parentJsonpFunction) parentJsonpFunction(data);
/******/
/******/ 		while(resolves.length) {
/******/ 			resolves.shift()();
/******/ 		}
/******/
/******/ 	};
/******/
/******/
/******/ 	// The module cache
/******/ 	var installedModules = {};
/******/
/******/ 	// object to store loaded and loading chunks
/******/ 	// undefined = chunk not loaded, null = chunk preloaded/prefetched
/******/ 	// Promise = chunk loading, 0 = chunk loaded
/******/ 	var installedChunks = {
/******/ 		"main": 0
/******/ 	};
/******/
/******/
/******/
/******/ 	// script path function
/******/ 	function jsonpScriptSrc(chunkId) {
/******/ 		return __webpack_require__.p + "" + chunkId + ".bootstrap.js"
/******/ 	}
/******/
/******/ 	// object to store loaded and loading wasm modules
/******/ 	var installedWasmModules = {};
/******/
/******/ 	function promiseResolve() { return Promise.resolve(); }
/******/
/******/ 	var wasmImportObjects = {
/******/ 		"../pkg/rayca_bg.wasm": function() {
/******/ 			return {
/******/ 				"./rayca_bg.js": {
/******/ 					"__wbg_context_new": function(p0i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_context_new"](p0i32);
/******/ 					},
/******/ 					"__wbindgen_object_drop_ref": function(p0i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbindgen_object_drop_ref"](p0i32);
/******/ 					},
/******/ 					"__wbg_log_cfe97bb2dbb8eba9": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_log_cfe97bb2dbb8eba9"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbindgen_cb_drop": function(p0i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbindgen_cb_drop"](p0i32);
/******/ 					},
/******/ 					"__wbg_new_693216e109162396": function() {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_new_693216e109162396"]();
/******/ 					},
/******/ 					"__wbg_stack_0ddaca5d1abfb52f": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_stack_0ddaca5d1abfb52f"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_error_09919627ac0992f5": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_error_09919627ac0992f5"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_instanceof_Window_a2a08d3918d7d4d0": function(p0i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_instanceof_Window_a2a08d3918d7d4d0"](p0i32);
/******/ 					},
/******/ 					"__wbg_document_14a383364c173445": function(p0i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_document_14a383364c173445"](p0i32);
/******/ 					},
/******/ 					"__wbg_fetch_23507368eed8d838": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_fetch_23507368eed8d838"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_getElementById_0c9415d96f5b9ec6": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_getElementById_0c9415d96f5b9ec6"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbindgen_object_clone_ref": function(p0i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbindgen_object_clone_ref"](p0i32);
/******/ 					},
/******/ 					"__wbindgen_string_new": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbindgen_string_new"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_newwithstrandinit_41c86e821f771b24": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_newwithstrandinit_41c86e821f771b24"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_instanceof_HtmlCanvasElement_7b561bd94e483f1d": function(p0i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_instanceof_HtmlCanvasElement_7b561bd94e483f1d"](p0i32);
/******/ 					},
/******/ 					"__wbg_setwidth_59ddc312219f205b": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_setwidth_59ddc312219f205b"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_setheight_70833966b4ed584e": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_setheight_70833966b4ed584e"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_getContext_b506f48cb166bf26": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_getContext_b506f48cb166bf26"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_clientWidth_ff949ad9c6d41cd2": function(p0i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_clientWidth_ff949ad9c6d41cd2"](p0i32);
/******/ 					},
/******/ 					"__wbg_clientHeight_a250dcf2e0afa47a": function(p0i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_clientHeight_a250dcf2e0afa47a"](p0i32);
/******/ 					},
/******/ 					"__wbg_instanceof_CanvasRenderingContext2d_9037c3eea625e27b": function(p0i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_instanceof_CanvasRenderingContext2d_9037c3eea625e27b"](p0i32);
/******/ 					},
/******/ 					"__wbg_putImageData_f71b039a7f3a0d8a": function(p0i32,p1i32,p2f64,p3f64) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_putImageData_f71b039a7f3a0d8a"](p0i32,p1i32,p2f64,p3f64);
/******/ 					},
/******/ 					"__wbg_newwithu8clampedarray_9c1ae19e8e194f7c": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_newwithu8clampedarray_9c1ae19e8e194f7c"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_instanceof_Response_e928c54c1025470c": function(p0i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_instanceof_Response_e928c54c1025470c"](p0i32);
/******/ 					},
/******/ 					"__wbg_arrayBuffer_9c26a73988618f92": function(p0i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_arrayBuffer_9c26a73988618f92"](p0i32);
/******/ 					},
/******/ 					"__wbg_newnoargs_fc5356289219b93b": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_newnoargs_fc5356289219b93b"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_call_4573f605ca4b5f10": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_call_4573f605ca4b5f10"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_new_306ce8d57919e6ae": function() {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_new_306ce8d57919e6ae"]();
/******/ 					},
/******/ 					"__wbg_self_ba1ddafe9ea7a3a2": function() {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_self_ba1ddafe9ea7a3a2"]();
/******/ 					},
/******/ 					"__wbg_window_be3cc430364fd32c": function() {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_window_be3cc430364fd32c"]();
/******/ 					},
/******/ 					"__wbg_globalThis_56d9c9f814daeeee": function() {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_globalThis_56d9c9f814daeeee"]();
/******/ 					},
/******/ 					"__wbg_global_8c35aeee4ac77f2b": function() {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_global_8c35aeee4ac77f2b"]();
/******/ 					},
/******/ 					"__wbindgen_is_undefined": function(p0i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbindgen_is_undefined"](p0i32);
/******/ 					},
/******/ 					"__wbg_instanceof_ArrayBuffer_a91000e6b0653ed1": function(p0i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_instanceof_ArrayBuffer_a91000e6b0653ed1"](p0i32);
/******/ 					},
/******/ 					"__wbg_call_9855a4612eb496cb": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_call_9855a4612eb496cb"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_now_513c8208bd94c09b": function() {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_now_513c8208bd94c09b"]();
/******/ 					},
/******/ 					"__wbg_new_78403b138428b684": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_new_78403b138428b684"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_resolve_f269ce174f88b294": function(p0i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_resolve_f269ce174f88b294"](p0i32);
/******/ 					},
/******/ 					"__wbg_then_1c698eedca15eed6": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_then_1c698eedca15eed6"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbg_then_4debc41d4fc92ce5": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_then_4debc41d4fc92ce5"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_buffer_de1150f91b23aa89": function(p0i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_buffer_de1150f91b23aa89"](p0i32);
/******/ 					},
/******/ 					"__wbg_new_97cf52648830a70d": function(p0i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_new_97cf52648830a70d"](p0i32);
/******/ 					},
/******/ 					"__wbg_set_a0172b213e2469e9": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_set_a0172b213e2469e9"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbg_length_e09c0b925ab8de5d": function(p0i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_length_e09c0b925ab8de5d"](p0i32);
/******/ 					},
/******/ 					"__wbg_set_b12cd0ab82903c2f": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbg_set_b12cd0ab82903c2f"](p0i32,p1i32,p2i32);
/******/ 					},
/******/ 					"__wbindgen_debug_string": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbindgen_debug_string"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbindgen_throw": function(p0i32,p1i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbindgen_throw"](p0i32,p1i32);
/******/ 					},
/******/ 					"__wbindgen_memory": function() {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbindgen_memory"]();
/******/ 					},
/******/ 					"__wbindgen_closure_wrapper579": function(p0i32,p1i32,p2i32) {
/******/ 						return installedModules["../pkg/rayca_bg.js"].exports["__wbindgen_closure_wrapper579"](p0i32,p1i32,p2i32);
/******/ 					}
/******/ 				}
/******/ 			};
/******/ 		},
/******/ 	};
/******/
/******/ 	// The require function
/******/ 	function __webpack_require__(moduleId) {
/******/
/******/ 		// Check if module is in cache
/******/ 		if(installedModules[moduleId]) {
/******/ 			return installedModules[moduleId].exports;
/******/ 		}
/******/ 		// Create a new module (and put it into the cache)
/******/ 		var module = installedModules[moduleId] = {
/******/ 			i: moduleId,
/******/ 			l: false,
/******/ 			exports: {}
/******/ 		};
/******/
/******/ 		// Execute the module function
/******/ 		modules[moduleId].call(module.exports, module, module.exports, __webpack_require__);
/******/
/******/ 		// Flag the module as loaded
/******/ 		module.l = true;
/******/
/******/ 		// Return the exports of the module
/******/ 		return module.exports;
/******/ 	}
/******/
/******/ 	// This file contains only the entry chunk.
/******/ 	// The chunk loading function for additional chunks
/******/ 	__webpack_require__.e = function requireEnsure(chunkId) {
/******/ 		var promises = [];
/******/
/******/
/******/ 		// JSONP chunk loading for javascript
/******/
/******/ 		var installedChunkData = installedChunks[chunkId];
/******/ 		if(installedChunkData !== 0) { // 0 means "already installed".
/******/
/******/ 			// a Promise means "currently loading".
/******/ 			if(installedChunkData) {
/******/ 				promises.push(installedChunkData[2]);
/******/ 			} else {
/******/ 				// setup Promise in chunk cache
/******/ 				var promise = new Promise(function(resolve, reject) {
/******/ 					installedChunkData = installedChunks[chunkId] = [resolve, reject];
/******/ 				});
/******/ 				promises.push(installedChunkData[2] = promise);
/******/
/******/ 				// start chunk loading
/******/ 				var script = document.createElement('script');
/******/ 				var onScriptComplete;
/******/
/******/ 				script.charset = 'utf-8';
/******/ 				script.timeout = 120;
/******/ 				if (__webpack_require__.nc) {
/******/ 					script.setAttribute("nonce", __webpack_require__.nc);
/******/ 				}
/******/ 				script.src = jsonpScriptSrc(chunkId);
/******/
/******/ 				// create error before stack unwound to get useful stacktrace later
/******/ 				var error = new Error();
/******/ 				onScriptComplete = function (event) {
/******/ 					// avoid mem leaks in IE.
/******/ 					script.onerror = script.onload = null;
/******/ 					clearTimeout(timeout);
/******/ 					var chunk = installedChunks[chunkId];
/******/ 					if(chunk !== 0) {
/******/ 						if(chunk) {
/******/ 							var errorType = event && (event.type === 'load' ? 'missing' : event.type);
/******/ 							var realSrc = event && event.target && event.target.src;
/******/ 							error.message = 'Loading chunk ' + chunkId + ' failed.\n(' + errorType + ': ' + realSrc + ')';
/******/ 							error.name = 'ChunkLoadError';
/******/ 							error.type = errorType;
/******/ 							error.request = realSrc;
/******/ 							chunk[1](error);
/******/ 						}
/******/ 						installedChunks[chunkId] = undefined;
/******/ 					}
/******/ 				};
/******/ 				var timeout = setTimeout(function(){
/******/ 					onScriptComplete({ type: 'timeout', target: script });
/******/ 				}, 120000);
/******/ 				script.onerror = script.onload = onScriptComplete;
/******/ 				document.head.appendChild(script);
/******/ 			}
/******/ 		}
/******/
/******/ 		// Fetch + compile chunk loading for webassembly
/******/
/******/ 		var wasmModules = {"0":["../pkg/rayca_bg.wasm"]}[chunkId] || [];
/******/
/******/ 		wasmModules.forEach(function(wasmModuleId) {
/******/ 			var installedWasmModuleData = installedWasmModules[wasmModuleId];
/******/
/******/ 			// a Promise means "currently loading" or "already loaded".
/******/ 			if(installedWasmModuleData)
/******/ 				promises.push(installedWasmModuleData);
/******/ 			else {
/******/ 				var importObject = wasmImportObjects[wasmModuleId]();
/******/ 				var req = fetch(__webpack_require__.p + "" + {"../pkg/rayca_bg.wasm":"9ff374c5e8b240834085"}[wasmModuleId] + ".module.wasm");
/******/ 				var promise;
/******/ 				if(importObject instanceof Promise && typeof WebAssembly.compileStreaming === 'function') {
/******/ 					promise = Promise.all([WebAssembly.compileStreaming(req), importObject]).then(function(items) {
/******/ 						return WebAssembly.instantiate(items[0], items[1]);
/******/ 					});
/******/ 				} else if(typeof WebAssembly.instantiateStreaming === 'function') {
/******/ 					promise = WebAssembly.instantiateStreaming(req, importObject);
/******/ 				} else {
/******/ 					var bytesPromise = req.then(function(x) { return x.arrayBuffer(); });
/******/ 					promise = bytesPromise.then(function(bytes) {
/******/ 						return WebAssembly.instantiate(bytes, importObject);
/******/ 					});
/******/ 				}
/******/ 				promises.push(installedWasmModules[wasmModuleId] = promise.then(function(res) {
/******/ 					return __webpack_require__.w[wasmModuleId] = (res.instance || res).exports;
/******/ 				}));
/******/ 			}
/******/ 		});
/******/ 		return Promise.all(promises);
/******/ 	};
/******/
/******/ 	// expose the modules object (__webpack_modules__)
/******/ 	__webpack_require__.m = modules;
/******/
/******/ 	// expose the module cache
/******/ 	__webpack_require__.c = installedModules;
/******/
/******/ 	// define getter function for harmony exports
/******/ 	__webpack_require__.d = function(exports, name, getter) {
/******/ 		if(!__webpack_require__.o(exports, name)) {
/******/ 			Object.defineProperty(exports, name, { enumerable: true, get: getter });
/******/ 		}
/******/ 	};
/******/
/******/ 	// define __esModule on exports
/******/ 	__webpack_require__.r = function(exports) {
/******/ 		if(typeof Symbol !== 'undefined' && Symbol.toStringTag) {
/******/ 			Object.defineProperty(exports, Symbol.toStringTag, { value: 'Module' });
/******/ 		}
/******/ 		Object.defineProperty(exports, '__esModule', { value: true });
/******/ 	};
/******/
/******/ 	// create a fake namespace object
/******/ 	// mode & 1: value is a module id, require it
/******/ 	// mode & 2: merge all properties of value into the ns
/******/ 	// mode & 4: return value when already ns object
/******/ 	// mode & 8|1: behave like require
/******/ 	__webpack_require__.t = function(value, mode) {
/******/ 		if(mode & 1) value = __webpack_require__(value);
/******/ 		if(mode & 8) return value;
/******/ 		if((mode & 4) && typeof value === 'object' && value && value.__esModule) return value;
/******/ 		var ns = Object.create(null);
/******/ 		__webpack_require__.r(ns);
/******/ 		Object.defineProperty(ns, 'default', { enumerable: true, value: value });
/******/ 		if(mode & 2 && typeof value != 'string') for(var key in value) __webpack_require__.d(ns, key, function(key) { return value[key]; }.bind(null, key));
/******/ 		return ns;
/******/ 	};
/******/
/******/ 	// getDefaultExport function for compatibility with non-harmony modules
/******/ 	__webpack_require__.n = function(module) {
/******/ 		var getter = module && module.__esModule ?
/******/ 			function getDefault() { return module['default']; } :
/******/ 			function getModuleExports() { return module; };
/******/ 		__webpack_require__.d(getter, 'a', getter);
/******/ 		return getter;
/******/ 	};
/******/
/******/ 	// Object.prototype.hasOwnProperty.call
/******/ 	__webpack_require__.o = function(object, property) { return Object.prototype.hasOwnProperty.call(object, property); };
/******/
/******/ 	// __webpack_public_path__
/******/ 	__webpack_require__.p = "";
/******/
/******/ 	// on error function for async loading
/******/ 	__webpack_require__.oe = function(err) { console.error(err); throw err; };
/******/
/******/ 	// object with all WebAssembly.instance exports
/******/ 	__webpack_require__.w = {};
/******/
/******/ 	var jsonpArray = window["webpackJsonp"] = window["webpackJsonp"] || [];
/******/ 	var oldJsonpFunction = jsonpArray.push.bind(jsonpArray);
/******/ 	jsonpArray.push = webpackJsonpCallback;
/******/ 	jsonpArray = jsonpArray.slice();
/******/ 	for(var i = 0; i < jsonpArray.length; i++) webpackJsonpCallback(jsonpArray[i]);
/******/ 	var parentJsonpFunction = oldJsonpFunction;
/******/
/******/
/******/ 	// Load entry module and return exports
/******/ 	return __webpack_require__(__webpack_require__.s = "./bootstrap.js");
/******/ })
/************************************************************************/
/******/ ({

/***/ "./bootstrap.js":
/*!**********************!*\
  !*** ./bootstrap.js ***!
  \**********************/
/*! no static exports found */
/***/ (function(module, exports, __webpack_require__) {

eval("// A dependency graph that contains any wasm must all be imported\n// asynchronously. This `bootstrap.js` file does the single async import, so\n// that no one else needs to worry about it again.\n__webpack_require__.e(/*! import() */ 0).then(__webpack_require__.bind(null, /*! ./index.js */ \"./index.js\"))\n  .catch(e => console.error(\"Error importing `index.js`:\", e));\n\n\n//# sourceURL=webpack:///./bootstrap.js?");

/***/ })

/******/ });