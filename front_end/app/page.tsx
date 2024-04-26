'use client';
import { Layout, Card, Upload } from 'antd';
import { PlusOutlined } from '@ant-design/icons';
import NavBar from './Componets/NavBar';

const { Header, Footer, Sider, Content } = Layout;

const headerStyle: React.CSSProperties = {
  textAlign: 'center',
  color: '#fff',
  height: 64,
  paddingInline: 0,
  lineHeight: '64px',
  backgroundColor: '#4096ff',
};

const contentStyle: React.CSSProperties = {
  display: 'flex',
  textAlign: 'center',
  height: '80%',
  position: "absolute", 
  width: "100%", 
  left: 0, 
  top: '10%',
  // minHeight: 120,
  // lineHeight: '120px',
  // color: '#fff',
  // backgroundColor: '#0958d9',
  backgroundColor: '#ffffff',
};

const siderStyle: React.CSSProperties = {
  textAlign: 'center',
  lineHeight: '120px',
  color: '#fff',
  display: 'none',
  backgroundColor: '#1677ff',
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
  return (
    <main style={{height:"100%", position:"absolute", width:"100%", left:0, top:0}}>
      <Header style={headerStyle}><NavBar/></Header>
        <Layout>
            <Sider width="25%" style={siderStyle}>
                Sider
            </Sider>
            <Content style={contentStyle}>
              <div style={{width: '50%'}}>
                <Card title='图片上传'>
                  <Upload listType='picture'>
                    <PlusOutlined />
                    <div style={{ marginTop: 8, color: '#666' }}>Upload Images</div>
                  </Upload>
                </Card>
              </div>
              <div>
                <p>
                  <a>123</a>
                </p>
                <p>
                  Content
                </p>
              </div>
            </Content>
        </Layout>
        <Footer style={footerStyle}>Footer</Footer>
    </main>
  );
}
