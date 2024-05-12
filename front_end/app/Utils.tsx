import axios from "axios";
import { UserSessionInfo } from "./Types";

// import {persistStore, persistReducer} from 'redux-persist';
// import storage from 'redux-persist/lib/storage';
// import { configureStore } from "@reduxjs/toolkit";
// import { combineReducers } from 'redux'
// import logger from 'redux-logger';

const saveSessionUserInfo = (props: UserSessionInfo) => {
    const { useremail } = props;
    if (useremail) {
        sessionStorage.setItem('useremail', useremail);
    }
};

const setAuthToken = (token: string | undefined) => {
    if(token){
        axios.defaults.headers.common['auth-token'] = token;
        sessionStorage.setItem('token', token);
    } else {
        delete axios.defaults.headers.common['auth-token'];
        sessionStorage.removeItem('token');
    }
}

// const parseAuthToken = (token: string) => {
//     if (token) {
//         return jwtDecode(token);
//     } else {
//         return null;
//     }
// }

// // redux-persist
// const persistConfig = {
//     key: 'root',
//     storage,
// };
// const allReducers = combineReducers({

// });
// const persistedReducer = persistReducer(persistConfig, allReducers);
// const store = configureStore({
//     reducer: persistedReducer,
//     middleware: (getDefaultMiddleware) => getDefaultMiddleware().concat(logger),
// });
// const persistor = persistStore(store);

// export { setAuthToken, parseAuthToken, saveSessionUserInfo, store, persistor };
export { setAuthToken, saveSessionUserInfo };