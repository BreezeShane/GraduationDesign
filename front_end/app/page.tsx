'use client';
import axios from 'axios';
import { Layout, Result, notification } from 'antd';
import NavBar from './Componets/NavBar';
import ContentPanel from '@/app/Pages/ContentPanel';
import { useEffect, useState } from 'react';
// import { Provider } from 'react-redux';
// import { PersistGate } from 'redux-persist/integration/react';
// import { store, persistor } from './Utils';

const { Header, Footer, Sider, Content } = Layout;

// Axios Global Config
axios.defaults.baseURL = `http://${process.env.BASE_URL}`;
axios.defaults.headers.post['Content-Type'] = 'application/x-www-form-urlencoded';
axios.defaults.headers.get['Content-Type'] = 'application/x-www-form-urlencoded';

const headerStyle: React.CSSProperties = {
  textAlign: 'center',
  color: '#fff',
  height: 64,
  paddingInline: 0,
  lineHeight: '64px',
  backgroundColor: '#FCFAF2',
  borderBottom: "solid",
  borderWidth: 0.25,
  borderColor: "#BDC0BA"
};

const layoutStyle: React.CSSProperties = {
  height: "100%",
  backgroundColor: "#ffffff",
};

const footerStyle: React.CSSProperties = {
  textAlign: 'center',
  position: "absolute",
  width: "100%",
  left: 0,
  top: '90%',
  color: '#fff',
  backgroundColor: '#4096ff',
};

export default function Home() {
  const [messageClient, contextHolder] = notification.useNotification();
  const [signStatus, setSignStatus] = useState(false);

  useEffect(() => {
    let token = sessionStorage.getItem('token');
    let useremail = sessionStorage.getItem('useremail')
    if ( token && useremail ){
      axios.defaults.headers.common['Authorization'] = token;
    }
  }, [])
  return (
    // <Provider store={store}>
      // <PersistGate loading={null} persistor={persistor}>
        <main style={{height:"100%", position:"absolute", width:"100%", left:0, top:0}}>
          {contextHolder}
          <Header style={headerStyle}>
            <NavBar messageClient={messageClient} signStatus={signStatus} setSignStatus={setSignStatus} />
          </Header>
          <div style={layoutStyle}>
              {
                !signStatus &&
                <Result
                  style={{
                    position: "absolute",
                    backgroundColor: "#ffffff",
                    zIndex: 999,
                    width: "100%",
                    height: "100%"
                  }}
                  title="Please click Sign Up or Sign In Button to start up!"
                />
              }
              {
                signStatus &&
                <ContentPanel signStatus={signStatus} messageClient={messageClient} />
              }
          </div>
          {/* <Footer style={footerStyle}>Footer</Footer> */}
        </main>
      // </PersistGate>
    // </Provider>
  );
}
