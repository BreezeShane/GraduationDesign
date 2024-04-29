import { Card, Upload } from 'antd';
import { PlusOutlined } from '@ant-design/icons';

const UserAdmin: React.FC = () => {
    return (
        <>
            <div style={{width: '50%'}}>
                <Card title='图片上传'>
                    <Upload listType='picture'>
                    <PlusOutlined />
                    <div style={{ marginTop: 8, color: '#666' }}>Upload Images</div>
                    </Upload>
                </Card>
                </div>
                <div style={{width: '50%'}}>
            </div>
            <div style={{ height: "10%" }}>
                <a id='SourceLink' href=''></a>
                </div>
                <div style={{ height: "90%" }}>
                <h1 id='SearchTitle'>Title</h1>
                <p id='SearchContent'>Content</p>
            </div>
        </>
    );
}

export default UserAdmin;