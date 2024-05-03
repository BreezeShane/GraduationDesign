import React, { useEffect, useRef, useState } from 'react';
import { Button, Space, Switch, Table, Input } from 'antd';
import type { TableColumnsType, TableProps, TablePaginationConfig, InputRef, TableColumnType, PaginationProps } from 'antd';
import { UnlockOutlined, LockOutlined, SearchOutlined } from "@ant-design/icons"
import axios from 'axios';
import type { FilterDropdownProps } from 'antd/es/table/interface';
import Highlighter from 'react-highlight-words';

type TableRowSelection<T> = TableProps<T>['rowSelection'];
type DataIndex = keyof DataType;

interface DataType {
  id: number,
  file_name: string,
  file_type: string,
  file_size: number,
  last_access_time: string,
  last_modified_time: string,
  creation_date: string,
}

const FileManage: React.FC<{ requestUrl: string }> = (props) => {
    const { requestUrl } = props;
    const [selected_row_keys, setSelectedRowKeys] = useState<React.Key[]>([]);
    const [lockButtons, setLockButtons] = useState(true);
    const [file_list, setFileList] = useState<DataType[]>([]);
    const [searchText, setSearchText] = useState('');
    const [searchedColumn, setSearchedColumn] = useState('');
    const searchInput = useRef<InputRef>(null);
    const [page_size, setPageSize] = useState(5);

    useEffect(() => {
      axios.get("/admin/model_manage",{
        params: {
          useremail: sessionStorage.getItem("useremail"),
          request_dir: requestUrl
        }
      }).then((res) => {
        if (res.status === 200) {
          let files: DataType[] = [];
          for (let idx in res.data) {
            let file = res.data[idx];
            file.id = idx;
            files.push(file);
          }
          setFileList(files);
        }
      }).catch((err) => {
        console.log("get files error: ", err)
      });
    }, []);

    const onSelectChange = (newSelectedRowKeys: React.Key[]) => {
      setSelectedRowKeys(newSelectedRowKeys);
    };

    const handleSwich = () => {
        setLockButtons(!lockButtons);
    }

    const rowSelection: TableRowSelection<DataType> = {
        selectedRowKeys: selected_row_keys,
        onChange: onSelectChange,
        selections: [
            Table.SELECTION_ALL,
            Table.SELECTION_INVERT,
            Table.SELECTION_NONE,
            {
                key: 'odd',
                text: 'Select Odd Row',
                onSelect: (changeableRowKeys) => {
                let newSelectedRowKeys = [];
                newSelectedRowKeys = changeableRowKeys.filter((_, index) => {
                    if (index % 2 !== 0) {
                    return false;
                    }
                    return true;
                });
                setSelectedRowKeys(newSelectedRowKeys);
                },
            },
            {
                key: 'even',
                text: 'Select Even Row',
                onSelect: (changeableRowKeys) => {
                let newSelectedRowKeys = [];
                newSelectedRowKeys = changeableRowKeys.filter((_, index) => {
                    if (index % 2 !== 0) {
                    return true;
                    }
                    return false;
                });
                setSelectedRowKeys(newSelectedRowKeys);
                },
            },
        ],
    };

    const handleBackupFiles = () => {
      let files2operate: any[] = [];
      for (let idx in selected_row_keys){
        let key: any = selected_row_keys[idx];
        files2operate.push(file_list[key].file_name);
      }
      axios.post("/admin/model_manage", {
        useremail: sessionStorage.getItem('useremail'),
        operation_type: "backup",
        files2operate: JSON.stringify(files2operate)
      })
    }

    const handleDeleteFile = () => {
      let files2operate = [];
      for (let idx in selected_row_keys){
        let key: any = selected_row_keys[idx];
        files2operate.push(file_list[key].file_name);
      }
      console.log(files2operate);
      axios.post("/admin/model_manage", {
        useremail: sessionStorage.getItem('useremail'),
        operation_type: "remove",
        files2operate: JSON.stringify(files2operate)
      })
    }

    const handleSearch = (selectedKeys: string[], confirm: FilterDropdownProps['confirm'], dataIndex: DataIndex) => {
      confirm();
      setSearchText(selectedKeys[0]);
      setSearchedColumn(dataIndex);
    };

    const handleReset = (clearFilters: () => void) => {
      clearFilters();
      setSearchText('');
    };

    const getColumnSearchProps = (dataIndex: DataIndex): TableColumnType<DataType> => ({
      filterDropdown: ({ setSelectedKeys, selectedKeys, confirm, clearFilters, close }) => (
        <div style={{ padding: 8 }} onKeyDown={(e) => e.stopPropagation()}>
          <Input
            ref={searchInput}
            placeholder={`Search ${dataIndex}`}
            value={selectedKeys[0]}
            onChange={(e) => setSelectedKeys(e.target.value ? [e.target.value] : [])}
            onPressEnter={() => handleSearch(selectedKeys as string[], confirm, dataIndex)}
            style={{ marginBottom: 8, display: 'block' }}
          />
          <Space>
            <Button
              type="primary"
              onClick={() => handleSearch(selectedKeys as string[], confirm, dataIndex)}
              icon={<SearchOutlined />}
              size="small"
              style={{ width: 90 }}
            >
              Search
            </Button>
            <Button
              onClick={() => clearFilters && handleReset(clearFilters)}
              size="small"
              style={{ width: 90 }}
            >
              Reset
            </Button>
            <Button
              type="link"
              size="small"
              onClick={() => {
                confirm({ closeDropdown: false });
                setSearchText((selectedKeys as string[])[0]);
                setSearchedColumn(dataIndex);
              }}
            >
              Filter
            </Button>
            <Button
              type="link"
              size="small"
              onClick={() => {
                close();
              }}
            >
              close
            </Button>
          </Space>
        </div>
      ),
      filterIcon: (filtered: boolean) => (
        <SearchOutlined style={{ color: filtered ? '#1677ff' : undefined }} />
      ),
      onFilter: (value, record) =>
        record[dataIndex]
          .toString()
          .toLowerCase()
          .includes((value as string).toLowerCase()),
      onFilterDropdownOpenChange: (visible) => {
        if (visible) {
          setTimeout(() => searchInput.current?.select(), 100);
        }
      },
      render: (text) =>
        searchedColumn === dataIndex ? (
          <Highlighter
            highlightStyle={{ backgroundColor: '#ffc069', padding: 0 }}
            searchWords={[searchText]}
            autoEscape
            textToHighlight={text ? text.toString() : ''}
          />
        ) : (
          text
        ),
    });

    const columns: TableColumnsType<DataType> = [
      {
        title: 'File Name',
        dataIndex: 'file_name',
        sorter: {
          compare: (a, b) => a.file_name.localeCompare(b.file_name),
          multiple: 4
        },
        sortDirections: ['descend', 'ascend'],
        ...getColumnSearchProps('file_name'),
      },
      {
        title: 'Type',
        dataIndex: 'file_type',
        sorter: {
          compare: (a, b) => a.file_type.localeCompare(b.file_type),
          multiple: 5
        },
        sortDirections: ['descend', 'ascend'],
        ...getColumnSearchProps('file_type'),
      },
      {
        title: 'Size',
        dataIndex: 'file_size',
        sorter: {
          compare: (a, b) => a.file_size - b.file_size,
          multiple: 6
        },
        sortDirections: ['descend', 'ascend'],
        // ...getColumnSearchProps('file_size'),
      },
      {
        title: 'Last Accessed Time',
        dataIndex: 'last_access_time',
        sorter: {
          compare: (a, b) => {
            let aDatetime = new Date(a.last_access_time).getTime();
            let bDatetime = new Date(b.last_access_time).getTime();
            return aDatetime - bDatetime;
          },
          multiple: 3
        },
        sortDirections: ['descend', 'ascend'],
        ...getColumnSearchProps('last_access_time'),
      },
      {
        title: 'Last Modified Time',
        dataIndex: 'last_modified_time',
        sorter: {
          compare: (a, b) => {
            let aDatetime = new Date(a.last_modified_time).getTime();
            let bDatetime = new Date(b.last_modified_time).getTime();
            return aDatetime - bDatetime;
          },
          multiple: 2
        },
        sortDirections: ['descend', 'ascend'],
        ...getColumnSearchProps('last_modified_time'),
      },
      {
        title: 'Creation date',
        dataIndex: 'creation_date',
        sorter: {
          compare: (a, b) => {
            let aDatetime = new Date(a.creation_date).getTime();
            let bDatetime = new Date(b.creation_date).getTime();
            return aDatetime - bDatetime;
          },
          multiple: 1
        },
        sortDirections: ['descend', 'ascend'],
        ...getColumnSearchProps('creation_date'),
      },
    ];

    const onShowSizeChange: PaginationProps['onShowSizeChange'] = (current, page_size) => {
      console.log(current, page_size);
      setPageSize(page_size)
    };

    const paginationProps: TablePaginationConfig = {
      position: ['bottomCenter'],
      pageSize: page_size,
      showSizeChanger: true,
      showQuickJumper: true,
      showTotal: (total, range) => `${range[0]}-${range[1]} of ${total} items`,
      pageSizeOptions: [5, 10, 13],
      onShowSizeChange
    }

    return (
        <>
          <Space align="center" style={{ float: "right" }}>
            <Switch
              checkedChildren={<LockOutlined />}
              unCheckedChildren={<UnlockOutlined />}
              checked={lockButtons}
              onChange={handleSwich}
            />
            <Button type="primary" size="middle" onClick={handleBackupFiles}>
                Back up
            </Button>
            <Button danger type="primary" disabled={lockButtons} size="middle" onClick={handleDeleteFile}>
                Delete
            </Button>
          </Space>
          <Table
            rowKey={item=>item.id}
            rowSelection={rowSelection}
            columns={columns}
            dataSource={file_list}
            pagination={paginationProps}
          />
        </>
    );
};

export default FileManage;