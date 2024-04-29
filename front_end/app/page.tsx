'use client';
import axios from 'axios';
import { Layout, notification } from 'antd';
import NavBar from './Componets/NavBar';
import ContentPanel from '@/app/Componets/ContentPanel';

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
  height: "79%",
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
  let token = sessionStorage.getItem('token');
  let useremail = sessionStorage.getItem('useremail');
  if (token && useremail){
    axios.defaults.headers.common['Authorization'] = token;
  }
  const [messageClient, contextHolder] = notification.useNotification();
  return (
    <main style={{height:"100%", position:"absolute", width:"100%", left:0, top:0}}>
      {contextHolder}
      <Header style={headerStyle}>
        <NavBar messageClient={messageClient}/>
      </Header>
      <div style={layoutStyle}>
          <ContentPanel messageClient={messageClient} />
      </div>
      <Footer style={footerStyle}>Footer</Footer>
    </main>
  );
}
